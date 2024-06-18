use std::collections::{BTreeMap, HashMap};
use std::num::ParseIntError;
use std::str::FromStr;
use std::sync::Arc;

use base64::prelude::*;
use base64::DecodeError;
use prost_types::value::Kind;
use prost_types::{value, Value};
use time::format_description::well_known::Rfc3339;
use time::macros::format_description;
use time::{Date, OffsetDateTime};

use google_cloud_googleapis::spanner::v1::struct_type::Field;
use google_cloud_googleapis::spanner::v1::StructType;

use crate::bigdecimal::{BigDecimal, ParseBigDecimalError};
use crate::value::CommitTimestamp;

#[derive(Clone)]
pub struct Row {
    index: Arc<HashMap<String, usize>>,
    fields: Arc<Vec<Field>>,
    values: Vec<Value>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Illegal Kind: field={0}, kind={1}")]
    KindMismatch(String, String),
    #[error("No kind found: field={0}")]
    NoKind(String),
    #[error("Parse field: field={0}")]
    IntParseError(String, #[source] ParseIntError),
    #[error("Failed to parse as Date|DateTime {0}")]
    DateParseError(String, #[source] time::error::Parse),
    #[error("Failed to parse as ByteArray {0}")]
    ByteParseError(String, #[source] DecodeError),
    #[error("Failed to parse as Struct name={0}, {1}")]
    StructParseError(String, &'static str),
    #[error("Failed to parse as Custom Type {0}")]
    CustomParseError(String),
    #[error("No column found: name={0}")]
    NoColumnFound(String),
    #[error("invalid column index: index={0}, length={1}")]
    InvalidColumnIndex(usize, usize),
    #[error("invalid struct column index: index={0}")]
    InvalidStructColumnIndex(usize),
    #[error("No column found in struct: name={0}")]
    NoColumnFoundInStruct(String),
    #[error("Failed to parse as BigDecimal field={0}")]
    BigDecimalParseError(String, #[source] ParseBigDecimalError),
    #[error("Failed to parse as Prost Timestamp field={0}")]
    ProstTimestampParseError(String, #[source] ::prost_types::TimestampError),
}

impl Row {
    pub fn new(index: Arc<HashMap<String, usize>>, fields: Arc<Vec<Field>>, values: Vec<Value>) -> Row {
        Row { index, fields, values }
    }

    pub fn column<T>(&self, column_index: usize) -> Result<T, Error>
    where
        T: TryFromValue,
    {
        column(&self.values, &self.fields, column_index)
    }

    pub fn column_by_name<T>(&self, column_name: &str) -> Result<T, Error>
    where
        T: TryFromValue,
    {
        self.column(index(&self.index, column_name)?)
    }
}

//don't use TryFrom trait to avoid the conflict
//https://github.com/rust-lang/rust/issues/50133
pub trait TryFromValue: Sized {
    fn try_from(value: &Value, field: &Field) -> Result<Self, Error>;
}

pub trait TryFromStruct: Sized {
    fn try_from_struct(s: Struct<'_>) -> Result<Self, Error>;
}

pub struct Struct<'a> {
    index: HashMap<String, usize>,
    metadata: &'a StructType,
    list_values: Option<&'a Vec<Value>>,
    struct_values: Option<&'a BTreeMap<String, Value>>,
}

impl<'a> Struct<'a> {
    pub fn new(metadata: &'a StructType, item: &'a Value, field: &'a Field) -> Result<Struct<'a>, Error> {
        let kind = as_ref(item, field)?;
        let mut index = HashMap::new();
        for (i, f) in metadata.fields.iter().enumerate() {
            index.insert(f.name.to_string(), i);
        }
        match kind {
            Kind::ListValue(s) => Ok(Struct {
                metadata,
                index,
                list_values: Some(&s.values),
                struct_values: None,
            }),
            Kind::StructValue(s) => Ok(Struct {
                metadata,
                index,
                list_values: None,
                struct_values: Some(&s.fields),
            }),
            _ => kind_to_error(kind, field),
        }
    }

    pub fn column<T>(&self, column_index: usize) -> Result<T, Error>
    where
        T: TryFromValue,
    {
        match self.list_values {
            Some(values) => column(values, &self.metadata.fields, column_index),
            None => match self.struct_values {
                Some(values) => {
                    let field = &self.metadata.fields[column_index];
                    let name = &field.name;
                    match values.get(name) {
                        Some(value) => T::try_from(value, field),
                        None => Err(Error::NoColumnFoundInStruct(name.to_string())),
                    }
                }
                None => Err(Error::InvalidStructColumnIndex(column_index)),
            },
        }
    }

    pub fn column_by_name<T>(&self, column_name: &str) -> Result<T, Error>
    where
        T: TryFromValue,
    {
        self.column(index(&self.index, column_name)?)
    }
}

impl TryFromValue for i64 {
    fn try_from(item: &Value, field: &Field) -> Result<Self, Error> {
        match as_ref(item, field)? {
            Kind::StringValue(s) => s.parse().map_err(|e| Error::IntParseError(field.name.to_string(), e)),
            v => kind_to_error(v, field),
        }
    }
}

impl TryFromValue for f64 {
    fn try_from(item: &Value, field: &Field) -> Result<Self, Error> {
        match as_ref(item, field)? {
            Kind::NumberValue(s) => Ok(*s),
            v => kind_to_error(v, field),
        }
    }
}

impl TryFromValue for bool {
    fn try_from(item: &Value, field: &Field) -> Result<Self, Error> {
        match as_ref(item, field)? {
            Kind::BoolValue(s) => Ok(*s),
            v => kind_to_error(v, field),
        }
    }
}

impl TryFromValue for OffsetDateTime {
    fn try_from(item: &Value, field: &Field) -> Result<Self, Error> {
        match as_ref(item, field)? {
            Kind::StringValue(s) => {
                Ok(OffsetDateTime::parse(s, &Rfc3339).map_err(|e| Error::DateParseError(field.name.to_string(), e))?)
            }
            v => kind_to_error(v, field),
        }
    }
}

impl TryFromValue for ::prost_types::Timestamp {
    fn try_from(item: &Value, field: &Field) -> Result<Self, Error> {
        match as_ref(item, field)? {
            Kind::StringValue(s) => Ok(::prost_types::Timestamp::from_str(s)
                .map_err(|e| Error::ProstTimestampParseError(field.name.to_string(), e))?),
            v => kind_to_error(v, field),
        }
    }
}

impl TryFromValue for CommitTimestamp {
    fn try_from(item: &Value, field: &Field) -> Result<Self, Error> {
        Ok(CommitTimestamp {
            timestamp: TryFromValue::try_from(item, field)?,
        })
    }
}

impl TryFromValue for Date {
    fn try_from(item: &Value, field: &Field) -> Result<Self, Error> {
        match as_ref(item, field)? {
            Kind::StringValue(s) => Date::parse(s, format_description!("[year]-[month]-[day]"))
                .map_err(|e| Error::DateParseError(field.name.to_string(), e)),
            v => kind_to_error(v, field),
        }
    }
}

impl TryFromValue for Vec<u8> {
    fn try_from(item: &Value, field: &Field) -> Result<Self, Error> {
        match as_ref(item, field)? {
            Kind::StringValue(s) => BASE64_STANDARD
                .decode(s)
                .map_err(|e| Error::ByteParseError(field.name.to_string(), e)),
            v => kind_to_error(v, field),
        }
    }
}

impl TryFromValue for BigDecimal {
    fn try_from(item: &Value, field: &Field) -> Result<Self, Error> {
        match as_ref(item, field)? {
            Kind::StringValue(s) => {
                Ok(BigDecimal::from_str(s).map_err(|e| Error::BigDecimalParseError(field.name.to_string(), e))?)
            }
            v => kind_to_error(v, field),
        }
    }
}

impl TryFromValue for String {
    fn try_from(item: &Value, field: &Field) -> Result<Self, Error> {
        match as_ref(item, field)? {
            Kind::StringValue(s) => Ok(s.to_string()),
            v => kind_to_error(v, field),
        }
    }
}

impl<T> TryFromValue for T
where
    T: TryFromStruct,
{
    fn try_from(item: &Value, field: &Field) -> Result<Self, Error> {
        let maybe_array = match field.r#type.as_ref() {
            None => return Err(Error::StructParseError(field.name.to_string(), "field type must not be none")),
            Some(tp) => tp.array_element_type.as_ref(),
        };
        let maybe_struct_type = match maybe_array {
            None => return Err(Error::StructParseError(field.name.to_string(), "array must not be none")),
            Some(tp) => tp.struct_type.as_ref(),
        };
        let struct_type = match maybe_struct_type {
            None => {
                return Err(Error::StructParseError(
                    field.name.to_string(),
                    "struct type in array must not be none ",
                ))
            }
            Some(struct_type) => struct_type,
        };

        T::try_from_struct(Struct::new(struct_type, item, field)?)
    }
}

impl<T> TryFromValue for Option<T>
where
    T: TryFromValue,
{
    fn try_from(item: &Value, field: &Field) -> Result<Self, Error> {
        match as_ref(item, field)? {
            Kind::NullValue(_i) => Ok(None),
            _ => Ok(Some(T::try_from(item, field)?)),
        }
    }
}

impl<T> TryFromValue for Vec<T>
where
    T: TryFromValue,
{
    fn try_from(item: &Value, field: &Field) -> Result<Self, Error> {
        match as_ref(item, field)? {
            Kind::ListValue(s) => s.values.iter().map(|v| T::try_from(v, field)).collect(),
            v => kind_to_error(v, field),
        }
    }
}

fn index(index: &HashMap<String, usize>, column_name: &str) -> Result<usize, Error> {
    match index.get(column_name) {
        Some(column_index) => Ok(*column_index),
        None => Err(Error::NoColumnFound(column_name.to_string())),
    }
}

fn column<T>(values: &[Value], fields: &[Field], column_index: usize) -> Result<T, Error>
where
    T: TryFromValue,
{
    if values.len() <= column_index {
        return Err(Error::InvalidColumnIndex(column_index, values.len()));
    }
    let value = &values[column_index];
    T::try_from(value, &fields[column_index])
}

pub fn as_ref<'a>(item: &'a Value, field: &'a Field) -> Result<&'a Kind, Error> {
    return match item.kind.as_ref() {
        Some(v) => Ok(v),
        None => Err(Error::NoKind(field.name.to_string())),
    };
}

pub fn kind_to_error<'a, T>(v: &'a value::Kind, field: &'a Field) -> Result<T, Error> {
    let actual = match v {
        Kind::StringValue(_s) => "StringValue".to_string(),
        Kind::BoolValue(_s) => "BoolValue".to_string(),
        Kind::NumberValue(_s) => "NumberValue".to_string(),
        Kind::ListValue(_s) => "ListValue".to_string(),
        Kind::StructValue(_s) => "StructValue".to_string(),
        _ => "unknown".to_string(),
    };
    Err(Error::KindMismatch(field.name.to_string(), actual))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::ops::Add;
    use std::str::FromStr;
    use std::sync::Arc;

    use prost_types::{Timestamp, Value};
    use time::OffsetDateTime;

    use google_cloud_googleapis::spanner::v1::struct_type::Field;

    use crate::bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive, Zero};
    use crate::row::{Error, Row, Struct as RowStruct, TryFromStruct};
    use crate::statement::{Kinds, ToKind, ToStruct, Types};
    use crate::value::CommitTimestamp;

    struct TestStruct {
        pub struct_field: String,
        pub struct_field_time: OffsetDateTime,
        pub commit_timestamp: CommitTimestamp,
        pub big_decimal: BigDecimal,
        pub prost_timestamp: Timestamp,
    }

    impl TryFromStruct for TestStruct {
        fn try_from_struct(s: RowStruct<'_>) -> Result<Self, Error> {
            Ok(TestStruct {
                struct_field: s.column_by_name("struct_field")?,
                struct_field_time: s.column_by_name("struct_field_time")?,
                commit_timestamp: s.column_by_name("commit_timestamp")?,
                big_decimal: s.column_by_name("big_decimal")?,
                prost_timestamp: s.column_by_name("prost_timestamp")?,
            })
        }
    }

    impl ToStruct for TestStruct {
        fn to_kinds(&self) -> Kinds {
            vec![
                ("struct_field", self.struct_field.to_kind()),
                ("struct_field_time", self.struct_field_time.to_kind()),
                // value from DB is timestamp. it's not string 'spanner.commit_timestamp()'.
                ("commit_timestamp", OffsetDateTime::from(self.commit_timestamp).to_kind()),
                ("big_decimal", self.big_decimal.to_kind()),
                ("prost_timestamp", self.prost_timestamp.to_kind()),
            ]
        }

        fn get_types() -> Types {
            vec![
                ("struct_field", String::get_type()),
                ("struct_field_time", OffsetDateTime::get_type()),
                ("commit_timestamp", CommitTimestamp::get_type()),
                ("big_decimal", BigDecimal::get_type()),
                ("prost_timestamp", Timestamp::get_type()),
            ]
        }
    }

    #[test]
    fn test_try_from() {
        let mut index = HashMap::new();
        index.insert("value".to_string(), 0);
        index.insert("array".to_string(), 1);
        index.insert("struct".to_string(), 2);
        index.insert("decimal".to_string(), 3);
        index.insert("timestamp".to_string(), 4);

        let now = OffsetDateTime::now_utc();
        let row = Row {
            index: Arc::new(index),
            fields: Arc::new(vec![
                Field {
                    name: "value".to_string(),
                    r#type: Some(String::get_type()),
                },
                Field {
                    name: "array".to_string(),
                    r#type: Some(Vec::<i64>::get_type()),
                },
                Field {
                    name: "struct".to_string(),
                    r#type: Some(Vec::<TestStruct>::get_type()),
                },
                Field {
                    name: "decimal".to_string(),
                    r#type: Some(BigDecimal::get_type()),
                },
                Field {
                    name: "timestamp".to_string(),
                    r#type: Some(Timestamp::get_type()),
                },
            ]),
            values: vec![
                Value {
                    kind: Some("aaa".to_kind()),
                },
                Value {
                    kind: Some(vec![10_i64, 100_i64].to_kind()),
                },
                // https://cloud.google.com/spanner/docs/query-syntax?hl=ja#using_structs_with_select
                // SELECT ARRAY(SELECT AS STRUCT * FROM TestStruct LIMIT 2) as struct
                Value {
                    kind: Some(
                        vec![
                            TestStruct {
                                struct_field: "aaa".to_string(),
                                struct_field_time: now,
                                commit_timestamp: CommitTimestamp { timestamp: now },
                                big_decimal: BigDecimal::from_str("-99999999999999999999999999999.999999999").unwrap(),
                                prost_timestamp: Timestamp::from_str("2024-01-01T01:13:45Z").unwrap(),
                            },
                            TestStruct {
                                struct_field: "bbb".to_string(),
                                struct_field_time: now,
                                commit_timestamp: CommitTimestamp { timestamp: now },
                                big_decimal: BigDecimal::from_str("99999999999999999999999999999.999999999").unwrap(),
                                prost_timestamp: Timestamp::from_str("2027-02-19T07:23:59Z").unwrap(),
                            },
                        ]
                        .to_kind(),
                    ),
                },
                Value {
                    kind: Some(BigDecimal::from_f64(100.999999999999).unwrap().to_kind()),
                },
                Value {
                    kind: Some(Timestamp::from_str("1999-12-31T23:59:59Z").unwrap().to_kind()),
                },
            ],
        };

        let value = row.column_by_name::<String>("value").unwrap();
        let array = row.column_by_name::<Vec<i64>>("array").unwrap();
        let struct_data = row.column_by_name::<Vec<TestStruct>>("struct").unwrap();
        let decimal = row.column_by_name::<BigDecimal>("decimal").unwrap();
        let ts = row.column_by_name::<Timestamp>("timestamp").unwrap();
        assert_eq!(value, "aaa");
        assert_eq!(array[0], 10);
        assert_eq!(array[1], 100);
        assert_eq!(decimal.to_f64().unwrap(), 100.999999999999);
        assert_eq!(format!("{ts:}"), "1999-12-31T23:59:59Z");
        assert_eq!(struct_data[0].struct_field, "aaa");
        assert_eq!(struct_data[0].struct_field_time, now);
        assert_eq!(
            struct_data[0].big_decimal,
            BigDecimal::from_str("-99999999999999999999999999999.999999999").unwrap()
        );
        assert_eq!(format!("{}", struct_data[0].prost_timestamp), "2024-01-01T01:13:45Z");
        assert_eq!(struct_data[1].struct_field, "bbb");
        assert_eq!(struct_data[1].struct_field_time, now);
        assert_eq!(struct_data[1].commit_timestamp.timestamp, now);
        assert_eq!(
            struct_data[1].big_decimal,
            BigDecimal::from_str("99999999999999999999999999999.999999999").unwrap()
        );
        assert_eq!(
            struct_data[1].big_decimal.clone().add(&struct_data[0].big_decimal),
            BigDecimal::zero()
        );
        assert_eq!(format!("{}", struct_data[1].prost_timestamp), "2027-02-19T07:23:59Z");
    }
}
