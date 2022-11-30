mod column;
mod query;
mod symbol;
mod table;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, ItemStruct};

#[proc_macro_derive(Table, attributes(column))]
pub fn table(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemStruct);
    let table = table::generate_table_methods(item.clone());
    let query = query::generate_query_methods(item);
    wrap_in_dummy_mod(quote! {
        #table
        #query
    })
}

#[proc_macro_derive(Query, attributes(column))]
pub fn query(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemStruct);
    let query = query::generate_query_methods(item);
    wrap_in_dummy_mod(query)
}

fn wrap_in_dummy_mod(item: impl ToTokens) -> TokenStream {
    //reference https://github.com/diesel-rs/diesel/blob/94599bdc86692900c888974bb4a03568799978d3/diesel_derives/src/util.rs
    let wrapped = quote! {
        #[allow(unused_imports)]
        const _: () = {
            use google_cloud_spanner::statement::{ToStruct, ToKind, Kinds, Types};
            use google_cloud_spanner::row::{Struct, TryFromStruct, Row, Error as RowError};

            #item
        };
    };
    wrapped.into()
}
