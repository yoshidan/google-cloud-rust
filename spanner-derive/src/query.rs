use quote::{quote, ToTokens};
use syn::ItemStruct;

use crate::column::Column;

pub(crate) fn generate_query_methods(item: ItemStruct) -> impl ToTokens {
    let struct_name = item.ident;

    let mut try_from_struct_fields = Vec::with_capacity(item.fields.len());
    for field in &item.fields {
        let field_var = field.ident.as_ref().unwrap();
        let column = Column::from(field);
        let column_name = column.name();
        try_from_struct_fields.push(quote! {
            #field_var: s.column_by_name(#column_name)?
        });
    }

    quote! {
        impl TryFromStruct for #struct_name {
            fn try_from_struct(s: Struct<'_>) -> Result<Self, RowError> {
                Ok(#struct_name {
                    #(
                        #try_from_struct_fields,
                    )*
                })
            }
        }

        impl TryFrom<Row> for #struct_name {
            type Error = RowError;
            fn try_from(s: Row) -> Result<Self, RowError> {
                Ok(#struct_name {
                    #(
                        #try_from_struct_fields,
                    )*
                })
            }
        }
    }
}
