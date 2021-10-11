use internal::spanner::v1::struct_type::Field;
use internal::spanner::v1::StructType;
use anyhow::{anyhow, Context, Result};
use chrono::{NaiveDate, NaiveDateTime, TimeZone, Utc};
use prost_types::value::Kind;
use prost_types::value::Kind::{StringValue, StructValue};
use prost_types::{value, ListValue, Value};
use std::collections::{BTreeMap, HashMap};
use std::ops::Deref;
use std::sync::Arc;

#[derive(Clone)]
pub struct Row {
    index: Arc<HashMap<String, usize>>,
    fields: Arc<Vec<Field>>,
    values: Vec<Value>,
}

impl Row {
    pub fn new(
        index: Arc<HashMap<String, usize>>,
        fields: Arc<Vec<Field>>,
        values: Vec<Value>,
    ) -> Row {
        Row {
            index,
            fields,
            values,
        }
    }

    pub fn column<T>(&self, column_index: usize) -> Result<T>
        where
            T: TryFromValue,
    {
        if self.values.len() <= column_index {
            return Err(anyhow!(
                "invalid column index: index={}, length={}",
                column_index,
                self.values.len()
            ));
        }
        if column_index < 0 {
            return Err(anyhow!(
                "invalid column index: index={}, length={}",
                column_index,
                self.values.len()
            ));
        }
        let value = &self.values[column_index];
        return T::try_from(value, &self.fields[column_index]);
    }

    pub fn column_by_name<T>(&self, column_name: &str) -> Result<T>
        where
            T: TryFromValue,
    {
        match self.index.get(column_name) {
            Some(column_index) => self.column(*column_index),
            None => Err(anyhow!("no column found: name={}", column_name)),
        }
    }
}

//don't use TryFrom trait to avoid the conflict
//https://github.com/rust-lang/rust/issues/50133
pub trait TryFromValue: Sized {
    fn try_from(value: &Value, field: &Field) -> Result<Self>;
}

pub trait TryFromStruct: Sized {
    fn try_from(s: Struct<'_>) -> Result<Self>;
}

pub struct Struct<'a> {
    index: HashMap<String, usize>,
    metadata: &'a StructType,
    values: &'a ListValue,
}

impl<'a> Struct<'a> {
    pub fn new(metadata: &'a StructType, values: &'a ListValue) -> Struct<'a> {
        let mut index = HashMap::new();
        for (i, f) in metadata.fields.iter().enumerate() {
            index.insert(f.name.clone(), i);
        }
        Struct {
            metadata,
            index,
            values,
        }
    }

    pub fn column_by_name<T>(&self, column_name: &str) -> Result<T>
        where
            T: TryFromValue,
    {
        let index = self.index[column_name];
        let field = &self.metadata.fields[index];
        let value = &self.values.values[index];
        return T::try_from(value, field);
    }
}

impl TryFromValue for i64 {
    fn try_from(item: &Value, field: &Field) -> Result<Self> {
        match item
            .kind
            .as_ref()
            .context(format!("{}: no kind found", field.name))?
        {
            Kind::StringValue(s) => match s.parse() {
                Ok(i) => Ok(i),
                Err(e) => Err(anyhow!("{}: i64 parse error {:?}", field.name, e)),
            },
            v => Err(anyhow!(
                "{}: i64: wanted=StringValue, actual={}",
                field.name,
                kind_to_string(v)
            )),
        }
    }
}

impl TryFromValue for f64 {
    fn try_from(item: &Value, field: &Field) -> Result<Self> {
        match item
            .kind
            .as_ref()
            .context(format!("{}: no kind found", field.name))?
        {
            Kind::NumberValue(s) => Ok(*s),
            v => Err(anyhow!(
                "{}: f64: wanted=NumberValue, actual={}",
                kind_to_string(v),
                field.name
            )),
        }
    }
}

impl TryFromValue for bool {
    fn try_from(item: &Value, field: &Field) -> Result<Self> {
        match item
            .kind
            .as_ref()
            .context(format!("{}: no kind found", field.name))?
        {
            Kind::BoolValue(s) => Ok(*s),
            v => Err(anyhow!(
                "{} bool: wanted=BoolValue, actual={}",
                kind_to_string(v),
                field.name
            )),
        }
    }
}

impl TryFromValue for NaiveDateTime {
    fn try_from(item: &Value, field: &Field) -> Result<Self> {
        match item
            .kind
            .as_ref()
            .context(format!("{}: no kind found", field.name))?
        {
            Kind::StringValue(s) => chrono::DateTime::parse_from_rfc3339(s)
                .map(|v| v.naive_utc())
                .context(format!("{}: datetime parse error ", field.name)),
            v => Err(anyhow!(
                "{} DateTime: wanted=StringValue, actual={}",
                kind_to_string(v),
                field.name
            )),
        }
    }
}

impl TryFromValue for NaiveDate {
    fn try_from(item: &Value, field: &Field) -> Result<Self> {
        match item
            .kind
            .as_ref()
            .context(format!("{}: no kind found", field.name))?
        {
            Kind::StringValue(s) => chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .context(format!("{}: date parse error ", field.name)),
            v => Err(anyhow!(
                "{} Date: wanted=StringValue, actual={}",
                kind_to_string(v),
                field.name
            )),
        }
    }
}

impl TryFromValue for Vec<u8> {
    fn try_from(item: &Value, field: &Field) -> Result<Self> {
        match item
            .kind
            .as_ref()
            .context(format!("{}: no kind found", field.name))?
        {
            Kind::StringValue(s) => base64::decode(s).map_err(|e| e.into()),
            v => Err(anyhow!(
                "{} Vec<u8>: wanted=StringValue, actual={}",
                kind_to_string(v),
                field.name
            )),
        }
    }
}

impl TryFromValue for String {
    fn try_from(item: &Value, field: &Field) -> Result<Self> {
        match item
            .kind
            .as_ref()
            .context(format!("{}: no kind found", field.name))?
        {
            Kind::StringValue(s) => Ok(s.to_string()),
            v => Err(anyhow!(
                "{} String: wanted=StringValue, actual={}",
                kind_to_string(v),
                field.name
            )),
        }
    }
}

impl<T> TryFromValue for T
    where
        T: TryFromStruct,
{
    fn try_from(item: &Value, field: &Field) -> Result<Self> {
        match item
            .kind
            .as_ref()
            .context(format!("{}: no kind found", field.name))?
        {
            Kind::ListValue(s) => {
                let maybe_array = match field.r#type.as_ref() {
                    None => return Err(anyhow!("field type must not be none {}", field.name)),
                    Some(tp) => tp.array_element_type.as_ref(),
                };
                let maybe_struct_type = match maybe_array {
                    None => return Err(anyhow!("array must not be none {}", field.name)),
                    Some(tp) => tp.struct_type.as_ref(),
                };
                let structured_value = match maybe_struct_type {
                    None => {
                        return Err(anyhow!(
                            "struct type in array must not be none {}",
                            field.name
                        ))
                    }
                    Some(struct_type) => Ok(Struct::new(struct_type, s)),
                };
                match structured_value {
                    Ok(v) => T::try_from(v),
                    Err(e) => Err(e),
                }
            }
            v => Err(anyhow!(
                "Struct: wanted=ListValue, actual={}",
                kind_to_string(v)
            )),
        }
    }
}

impl<T> TryFromValue for Option<T>
    where
        T: TryFromValue,
{
    fn try_from(item: &Value, field: &Field) -> Result<Self> {
        match item
            .kind
            .as_ref()
            .context(format!("{}: no kind found", field.name))?
        {
            Kind::NullValue(i) => Ok(None),
            _ => Ok(Some(T::try_from(item, field)?)),
        }
    }
}

impl<T> TryFromValue for Vec<T>
    where
        T: TryFromValue,
{
    fn try_from(item: &Value, field: &Field) -> Result<Self> {
        match item
            .kind
            .as_ref()
            .context(format!("{}: no kind found", field.name))?
        {
            Kind::ListValue(s) => s.values.iter().map(|v| T::try_from(v, field)).collect(),
            v => Err(anyhow!(
                "Vec<T>: wanted=ListValue, actual={}",
                kind_to_string(v)
            )),
        }
    }
}

fn kind_to_string(v: &value::Kind) -> String {
    return match v {
        Kind::StringValue(s) => "StringValue".to_string(),
        Kind::BoolValue(s) => "BoolValue".to_string(),
        Kind::NumberValue(s) => "NumberValue".to_string(),
        Kind::ListValue(s) => "ListValue".to_string(),
        Kind::StructValue(s) => "StructValue".to_string(),
        _ => "unknown".to_string(),
    };
}
