use prost_types::{ListValue, Value};

use crate::key::KeySet;
use crate::statement::{ToKind, ToStruct};
use google_cloud_googleapis::spanner::v1::mutation::{Delete, Operation, Write};
use google_cloud_googleapis::spanner::v1::Mutation;

fn write(table: &str, columns: &[&str], values: &[&dyn ToKind]) -> Write {
    let values = values
        .iter()
        .map(|x| Value {
            kind: Some(x.to_kind()),
        })
        .collect();

    Write {
        table: table.to_string(),
        columns: columns.iter().map(|x| x.to_string()).collect(),
        values: vec![ListValue { values }],
    }
}

fn write_map(table: &str, columns_ans_values: &[(&str, &dyn ToKind)]) -> Write {
    let mut columns = Vec::with_capacity(columns_ans_values.len());
    let mut values = Vec::with_capacity(columns_ans_values.len());
    columns_ans_values.into_iter().for_each(|x| {
        columns.push(x.0.to_string());
        values.push(Value {
            kind: Some(x.1.to_kind()),
        })
    });
    Write {
        table: table.to_string(),
        columns,
        values: vec![ListValue { values }],
    }
}

fn write_struct(table: &str, to_struct: impl ToStruct) -> Write {
    let kinds = to_struct.to_kinds();
    let mut columns = Vec::with_capacity(kinds.len());
    let mut values = Vec::with_capacity(kinds.len());
    kinds.into_iter().for_each(|x| {
        columns.push(x.0.to_string());
        values.push(Value { kind: Some(x.1) })
    });
    Write {
        table: table.to_string(),
        columns,
        values: vec![ListValue { values }],
    }
}

/// Insert returns a Mutation to insert a row into a table. If the row already
/// exists, the write or transaction fails with codes.AlreadyExists.
pub fn insert(table: &str, columns: &[&str], values: &[&dyn ToKind]) -> Mutation {
    Mutation {
        operation: Some(Operation::Insert(write(table, columns, values))),
    }
}

/// insert_map returns a Mutation to insert a row into a table, specified by
/// a map of column name to value. If the row already exists, the write or
/// transaction fails with codes.AlreadyExists.
pub fn insert_map(table: &str, columns_ans_values: &[(&str, &dyn ToKind)]) -> Mutation {
    Mutation {
        operation: Some(Operation::Insert(write_map(table, columns_ans_values))),
    }
}

/// insert_struct returns a Mutation to insert a row into a table, specified by
/// a Rust struct.  If the row already exists, the write or transaction fails with
/// codes.AlreadyExists.
pub fn insert_struct(table: &str, to_struct: impl ToStruct) -> Mutation {
    Mutation {
        operation: Some(Operation::Insert(write_struct(table, to_struct))),
    }
}

/// update returns a Mutation to update a row in a table. If the row does not
/// already exist, the write or transaction fails.
pub fn update(table: &str, columns: &[&str], values: &[&dyn ToKind]) -> Mutation {
    Mutation {
        operation: Some(Operation::Update(write(table, columns, values))),
    }
}

/// update_map returns a Mutation to update a row in a table, specified by
/// a map of column to value. If the row does not already exist, the write or
/// transaction fails.
pub fn update_map(table: &str, columns_ans_values: &[(&str, &dyn ToKind)]) -> Mutation {
    Mutation {
        operation: Some(Operation::Update(write_map(table, columns_ans_values))),
    }
}

/// update_struct returns a Mutation to update a row in a table, specified by a Go
/// struct. If the row does not already exist, the write or transaction fails.
pub fn update_struct(table: &str, to_struct: impl ToStruct) -> Mutation {
    Mutation {
        operation: Some(Operation::Update(write_struct(table, to_struct))),
    }
}

/// replace returns a Mutation to insert a row into a table, deleting any
/// existing row. Unlike InsertOrUpdate, this means any values not explicitly
/// written become NULL.
///
/// For a similar example, See Update.
pub fn replace(table: &str, columns: &[&str], values: &[&dyn ToKind]) -> Mutation {
    Mutation {
        operation: Some(Operation::Replace(write(table, columns, values))),
    }
}

/// replace_map returns a Mutation to insert a row into a table, deleting any
/// existing row. Unlike InsertOrUpdateMap, this means any values not explicitly
/// written become NULL.  The row is specified by a map of column to value.
///
/// For a similar example, See update_map.
pub fn replace_map(table: &str, columns_ans_values: &[(&str, &dyn ToKind)]) -> Mutation {
    Mutation {
        operation: Some(Operation::Replace(write_map(table, columns_ans_values))),
    }
}

/// replace_struct returns a Mutation to insert a row into a table, deleting any existing row.
///
/// For a similar example, See update_struct.
pub fn replace_struct(table: &str, to_struct: impl ToStruct) -> Mutation {
    Mutation {
        operation: Some(Operation::Replace(write_struct(table, to_struct))),
    }
}

/// insert_or_update returns a Mutation to insert a row into a table. If the row
/// already exists, it updates it instead. Any column values not explicitly
/// written are preserved.
///
/// For a similar example, See update.
pub fn insert_or_update(table: &str, columns: &[&str], values: &[&dyn ToKind]) -> Mutation {
    Mutation {
        operation: Some(Operation::InsertOrUpdate(write(table, columns, values))),
    }
}

/// insert_or_update returns a Mutation to insert a row into a table. If the row
/// already exists, it updates it instead. Any column values not explicitly
/// written are preserved.
///
/// For a similar example, See update.
pub fn insert_or_update_map(table: &str, columns_ans_values: &[(&str, &dyn ToKind)]) -> Mutation {
    Mutation {
        operation: Some(Operation::InsertOrUpdate(write_map(
            table,
            columns_ans_values,
        ))),
    }
}

/// insert_or_update_struct returns a Mutation to insert a row into a table,
/// specified by a Go struct. If the row already exists, it updates it instead.
/// Any column values not explicitly written are preserved.
/// For a similar example, See update_struct.
pub fn insert_or_update_struct(table: &str, to_struct: impl ToStruct) -> Mutation {
    Mutation {
        operation: Some(Operation::InsertOrUpdate(write_struct(table, to_struct))),
    }
}

/// delete removes the rows described by the KeySet from the table. It succeeds
/// whether or not the keys were present.
pub fn delete(table: &str, key_set: impl Into<KeySet>) -> Mutation {
    Mutation {
        operation: Some(Operation::Delete(Delete {
            table: table.to_string(),
            key_set: Some(key_set.into().inner),
        })),
    }
}

#[cfg(test)]
mod tests {
    use crate::key::*;
    use crate::mutation::*;
    use crate::statement::{Kinds, ToKind, Types};
    use crate::value::CommitTimestamp;

    use google_cloud_googleapis::spanner::*;
    use prost_types::value::Kind;

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
            &["GuildId", "UserId", "UpdatedAt"],
            &[&"1", &"2", &CommitTimestamp::new()],
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
            &[
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
            &["GuildId", "UserId", "UpdatedAt"],
            &[&"1", &"2", &CommitTimestamp::new()],
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
            &["GuildId", "UserId", "UpdatedAt"],
            &[&"1", &"2", &CommitTimestamp::new()],
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
            &["GuildId", "UserId", "UpdatedAt"],
            &[&"1", &"2", &CommitTimestamp::new()],
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
