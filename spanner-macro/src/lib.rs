mod table;
mod symbol;
mod util;
mod query;
mod column;

use proc_macro::{TokenStream};

#[proc_macro_derive(Table, attributes(column))]
pub fn table(input: TokenStream) -> TokenStream {
    table::generate_table_methods(input)
}