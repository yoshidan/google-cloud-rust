use prost_types::value::Kind;
use prost_types::{ListValue, Value};

use crate::statement::{ToKind, ToStruct};
use google_cloud_googleapis::spanner::v1::mutation::{Delete, Operation, Write};
use google_cloud_googleapis::spanner::v1::{KeySet, Mutation};

fn write<T, C>(table: T, columns: Vec<C>, values: Vec<Kind>) -> Write
where
    T: Into<String>,
    C: Into<String>,
{
    let values = values
        .into_iter()
        .map(|x| Value { kind: Some(x) })
        .collect();

    Write {
        table: table.into(),
        columns: columns.into_iter().map(|x| x.into()).collect(),
        values: vec![ListValue { values }],
    }
}

/// Insert returns a Mutation to insert a row into a table. If the row already
/// exists, the write or transaction fails with codes.AlreadyExists.
pub fn insert<T, C>(table: T, columns: Vec<C>, values: Vec<Kind>) -> Mutation
where
    T: Into<String>,
    C: Into<String>,
{
    Mutation {
        operation: Some(Operation::Insert(write(table, columns, values))),
    }
}

/// insert_map returns a Mutation to insert a row into a table, specified by
/// a map of column name to value. If the row already exists, the write or
/// transaction fails with codes.AlreadyExists.
pub fn insert_map<T, C>(table: T, columns_ans_values: Vec<(C, &(dyn ToKind))>) -> Mutation
where
    T: Into<String>,
    C: Into<String>,
{
    let (columns, values) = map_to_columns_values(columns_ans_values);
    insert(table, columns, values)
}

/// insert_struct returns a Mutation to insert a row into a table, specified by
/// a Rust struct.  If the row already exists, the write or transaction fails with
/// codes.AlreadyExists.
pub fn insert_struct<T>(table: T, to_struct: impl ToStruct) -> Mutation
where
    T: Into<String>,
{
    let (columns, values) = struct_to_columns_values(&to_struct);
    insert(table, columns, values)
}

/// update returns a Mutation to update a row in a table. If the row does not
/// already exist, the write or transaction fails.
pub fn update<T, C>(table: T, columns: Vec<C>, values: Vec<Kind>) -> Mutation
where
    T: Into<String>,
    C: Into<String>,
{
    Mutation {
        operation: Some(Operation::Update(write(table, columns, values))),
    }
}

/// update_map returns a Mutation to update a row in a table, specified by
/// a map of column to value. If the row does not already exist, the write or
/// transaction fails.
pub fn update_map<T, C>(table: T, columns_ans_values: Vec<(C, &(dyn ToKind))>) -> Mutation
where
    T: Into<String>,
    C: Into<String>,
{
    let (columns, values) = map_to_columns_values(columns_ans_values);
    update(table, columns, values)
}

/// update_struct returns a Mutation to update a row in a table, specified by a Go
/// struct. If the row does not already exist, the write or transaction fails.
pub fn update_struct<T>(table: T, to_struct: impl ToStruct) -> Mutation
where
    T: Into<String>,
{
    let (columns, values) = struct_to_columns_values(&to_struct);
    update(table, columns, values)
}

/// replace returns a Mutation to insert a row into a table, deleting any
/// existing row. Unlike InsertOrUpdate, this means any values not explicitly
/// written become NULL.
///
/// For a similar example, See Update.
pub fn replace<T, C>(table: T, columns: Vec<C>, values: Vec<Kind>) -> Mutation
where
    T: Into<String>,
    C: Into<String>,
{
    Mutation {
        operation: Some(Operation::Replace(write(table, columns, values))),
    }
}

/// replace_map returns a Mutation to insert a row into a table, deleting any
/// existing row. Unlike InsertOrUpdateMap, this means any values not explicitly
/// written become NULL.  The row is specified by a map of column to value.
///
/// For a similar example, See update_map.
pub fn replace_map<T, C>(table: T, columns_ans_values: Vec<(C, &(dyn ToKind))>) -> Mutation
where
    T: Into<String>,
    C: Into<String>,
{
    let (columns, values) = map_to_columns_values(columns_ans_values);
    replace(table, columns, values)
}

/// replace_struct returns a Mutation to insert a row into a table, deleting any existing row.
///
/// For a similar example, See update_struct.
pub fn replace_struct<T>(table: T, to_struct: impl ToStruct) -> Mutation
where
    T: Into<String>,
{
    let (columns, values) = struct_to_columns_values(&to_struct);
    replace(table, columns, values)
}

/// insert_or_update returns a Mutation to insert a row into a table. If the row
/// already exists, it updates it instead. Any column values not explicitly
/// written are preserved.
///
/// For a similar example, See update.
pub fn insert_or_update<T, C>(table: T, columns: Vec<C>, values: Vec<Kind>) -> Mutation
where
    T: Into<String>,
    C: Into<String>,
{
    Mutation {
        operation: Some(Operation::InsertOrUpdate(write(table, columns, values))),
    }
}

/// insert_or_update_struct returns a Mutation to insert a row into a table,
/// specified by a Go struct. If the row already exists, it updates it instead.
/// Any column values not explicitly written are preserved.
/// For a similar example, See update_struct.
pub fn insert_or_update_struct<T>(table: T, to_struct: impl ToStruct) -> Mutation
where
    T: Into<String>,
{
    let (columns, values) = struct_to_columns_values(&to_struct);
    insert_or_update(table, columns, values)
}

/// delete removes the rows described by the KeySet from the table. It succeeds
/// whether or not the keys were present.
pub fn delete<T: Into<String>, F: Into<KeySet>>(table: T, key_set: F) -> Mutation {
    Mutation {
        operation: Some(Operation::Delete(Delete {
            table: table.into(),
            key_set: Some(key_set.into()),
        })),
    }
}

fn struct_to_columns_values<'a>(to_struct: impl ToStruct) -> (Vec<&'a str>, Vec<Kind>) {
    let kind = to_struct.to_kinds();
    let columns = kind.iter().map(|x| x.0).collect();
    let values = kind.into_iter().map(|x| x.1).collect();
    (columns, values)
}

fn map_to_columns_values<T: Into<String>>(
    columns_ans_values: Vec<(T, &(dyn ToKind))>,
) -> (Vec<T>, Vec<Kind>) {
    let mut columns = Vec::with_capacity(columns_ans_values.len());
    let mut values = Vec::with_capacity(columns_ans_values.len());
    columns_ans_values.into_iter().for_each(|x| {
        columns.push(x.0);
        values.push(x.1.to_kind())
    });
    (columns, values)
}

#[cfg(test)]
mod tests {
    use crate::key::*;
    use crate::mutation::*;
    use crate::statement::{Kinds, ToKind, Types};
    use crate::value::CommitTimestamp;
    use chrono::Utc;
    use google_cloud_googleapis::spanner::*;

    struct TestStruct {
        pub struct_field: String,
    }

    impl ToStruct for TestStruct {
        fn to_kinds(&self) -> Kinds {
            vec![("StructField", self.struct_field.to_kind())]
        }

        fn get_types() -> Types {
            vec![("StructField", String::get_type())]
        }
    }

    #[test]
    fn test_insert() {
        let mutation = insert(
            "Guild",
            vec!["GuildId", "UserId", "UpdatedAt"],
            vec![
                "1".to_kind(),
                "2".to_kind(),
                CommitTimestamp {
                    timestamp: Utc::now(),
                }
                .to_kind(),
            ],
        );
        match mutation.operation.unwrap() {
            v1::mutation::Operation::Insert(mut w) => {
                assert_eq!("Guild", w.table);
                assert_eq!(3, w.values.pop().unwrap().values.len());
                assert_eq!("UpdatedAt", w.columns.pop().unwrap());
                assert_eq!("UserId", w.columns.pop().unwrap());
                assert_eq!("GuildId", w.columns.pop().unwrap());
            }
            _ => panic!("invalid operation"),
        }
    }

    #[test]
    fn test_insert_map() {
        let user_id = 1;
        let mutation = insert_map(
            "Guild",
            vec![
                ("UserId", &"aa"),
                ("GuildId", &user_id),
                ("updatedAt", &CommitTimestamp::new()),
            ],
        );
        match mutation.operation.unwrap() {
            v1::mutation::Operation::Insert(mut w) => {
                assert_eq!("Guild", w.table);
                assert_eq!(3, w.values.pop().unwrap().values.len());
                assert_eq!(3, w.columns.len());
            }
            _ => panic!("invalid operation"),
        }
    }

    #[test]
    fn test_insert_struct() {
        let mutation = insert_struct(
            "Guild",
            TestStruct {
                struct_field: "abc".to_string(),
            },
        );
        match mutation.operation.unwrap() {
            v1::mutation::Operation::Insert(w) => assert_struct(w),
            _ => panic!("invalid operation"),
        }
    }

    #[test]
    fn test_insert_struct_ref() {
        let mutation = insert_struct(
            "Guild",
            &TestStruct {
                struct_field: "abc".to_string(),
            },
        );
        match mutation.operation.unwrap() {
            v1::mutation::Operation::Insert(w) => assert_struct(w),
            _ => panic!("invalid operation"),
        }
    }

    #[test]
    fn test_update() {
        let mutation = update(
            "Guild",
            vec!["GuildId", "UserId", "UpdatedAt"],
            vec![
                "1".to_kind(),
                "2".to_kind(),
                CommitTimestamp {
                    timestamp: Utc::now(),
                }
                .to_kind(),
            ],
        );
        match mutation.operation.unwrap() {
            v1::mutation::Operation::Update(mut w) => {
                assert_eq!("Guild", w.table);
                assert_eq!(3, w.values.pop().unwrap().values.len());
                assert_eq!(3, w.columns.len());
            }
            _ => panic!("invalid operation"),
        }
    }

    #[test]
    fn test_update_struct() {
        let mutation = update_struct(
            "Guild",
            TestStruct {
                struct_field: "abc".to_string(),
            },
        );
        match mutation.operation.unwrap() {
            v1::mutation::Operation::Update(w) => assert_struct(w),
            _ => panic!("invalid operation"),
        }
    }

    #[test]
    fn test_update_struct_ref() {
        let st = TestStruct {
            struct_field: "abc".to_string(),
        };
        let mutation = update_struct("Guild", &st);
        match mutation.operation.unwrap() {
            v1::mutation::Operation::Update(w) => assert_struct(w),
            _ => panic!("invalid operation"),
        }
    }

    #[test]
    fn test_replace() {
        let mutation = replace(
            "Guild",
            vec!["GuildId", "UserId", "UpdatedAt"],
            vec![
                "1".to_kind(),
                "2".to_kind(),
                CommitTimestamp {
                    timestamp: Utc::now(),
                }
                .to_kind(),
            ],
        );
        match mutation.operation.unwrap() {
            v1::mutation::Operation::Replace(mut w) => {
                assert_eq!("Guild", w.table);
                assert_eq!(3, w.values.pop().unwrap().values.len());
                assert_eq!(3, w.columns.len());
            }
            _ => panic!("invalid operation"),
        }
    }

    #[test]
    fn test_replace_struct() {
        let mutation = replace_struct(
            "Guild",
            TestStruct {
                struct_field: "abc".to_string(),
            },
        );
        match mutation.operation.unwrap() {
            v1::mutation::Operation::Replace(w) => assert_struct(w),
            _ => panic!("invalid operation"),
        }
    }

    #[test]
    fn test_insert_or_update() {
        let mutation = insert_or_update(
            "Guild",
            vec!["GuildId", "UserId", "UpdatedAt"],
            vec![
                "1".to_kind(),
                "2".to_kind(),
                CommitTimestamp {
                    timestamp: Utc::now(),
                }
                .to_kind(),
            ],
        );
        match mutation.operation.unwrap() {
            v1::mutation::Operation::InsertOrUpdate(mut w) => {
                assert_eq!("Guild", w.table);
                assert_eq!(3, w.values.pop().unwrap().values.len());
                assert_eq!(3, w.columns.len());
            }
            _ => panic!("invalid operation"),
        }
    }

    #[test]
    fn test_insert_or_update_struct() {
        let mutation = insert_or_update_struct(
            "Guild",
            TestStruct {
                struct_field: "abc".to_string(),
            },
        );
        match mutation.operation.unwrap() {
            v1::mutation::Operation::InsertOrUpdate(w) => assert_struct(w),
            _ => panic!("invalid operation"),
        }
    }

    #[test]
    fn test_delete() {
        let mutation = delete("Guild", all_keys());
        match mutation.operation.unwrap() {
            v1::mutation::Operation::Delete(w) => {
                assert_eq!("Guild", w.table);
                assert_eq!(true, w.key_set.unwrap().all);
            }
            _ => panic!("invalid operation"),
        }
    }

    fn assert_struct(mut w: Write) {
        assert_eq!("Guild", w.table);
        assert_eq!("StructField", w.columns.pop().unwrap());
        assert_eq!(
            "abc",
            match w.values.pop().unwrap().values.pop().unwrap().kind.unwrap() {
                Kind::StringValue(v) => v,
                _ => panic!("error"),
            }
        );
    }
}
