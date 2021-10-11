use internal::spanner::v1::mutation::{Delete, Operation, Write};
use internal::spanner::v1::{KeySet, Mutation};
use prost_types::value::Kind;
use prost_types::value::Kind::StringValue;
use prost_types::{value, ListValue, Value};

pub fn write<T, C>(table: T, columns: Vec<C>, values: Vec<Kind>) -> Write
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

pub fn insert<T, C>(table: T, columns: Vec<C>, values: Vec<Kind>) -> Mutation
where
    T: Into<String>,
    C: Into<String>,
{
    Mutation {
        operation: Some(Operation::Insert(write(table, columns, values))),
    }
}

pub fn update<T, C>(table: T, columns: Vec<C>, values: Vec<Kind>) -> Mutation
where
    T: Into<String>,
    C: Into<String>,
{
    Mutation {
        operation: Some(Operation::Update(write(table, columns, values))),
    }
}

pub fn replace<T, C>(table: T, columns: Vec<C>, values: Vec<Kind>) -> Mutation
where
    T: Into<String>,
    C: Into<String>,
{
    Mutation {
        operation: Some(Operation::Replace(write(table, columns, values))),
    }
}

pub fn insert_or_update<T, C>(table: T, columns: Vec<C>, values: Vec<Kind>) -> Mutation
where
    T: Into<String>,
    C: Into<String>,
{
    Mutation {
        operation: Some(Operation::InsertOrUpdate(write(table, columns, values))),
    }
}

pub fn delete<T: Into<String>, F: Into<KeySet>>(table: T, key_set: F) -> Mutation {
    Mutation {
        operation: Some(Operation::Delete(Delete {
            table: table.into(),
            key_set: Some(key_set.into()),
        })),
    }
}
