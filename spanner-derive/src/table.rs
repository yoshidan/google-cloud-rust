use quote::{quote, ToTokens};
use syn::ItemStruct;

use crate::column::Column;

pub(crate) fn generate_table_methods(item: ItemStruct) -> impl ToTokens {
    let struct_name = item.ident;

    let mut to_kinds_fields = Vec::with_capacity(item.fields.len());
    let mut get_types_fields = Vec::with_capacity(item.fields.len());
    for field in &item.fields {
        let field_var = field.ident.as_ref().unwrap();
        let column = Column::from(field);
        let column_name = column.name();
        let ty = &field.ty;
        let mut get_field_type = quote! { <#ty> };
        let mut to_kind_field_type = quote! { self.#field_var };
        if column.commit_timestamp {
            get_field_type = quote! { CommitTimestamp };
            to_kind_field_type = quote! { CommitTimestamp::new() };
        }
        to_kinds_fields.push(quote! {
            (#column_name, #to_kind_field_type.to_kind())
        });
        get_types_fields.push(quote! {
            (#column_name, #get_field_type::get_type())
        });
    }

    quote! {

        impl ToStruct for #struct_name  {

            fn to_kinds(&self) -> Kinds {
                vec![
                    #(
                        #to_kinds_fields,
                    )*
                ]
            }
            fn get_types() -> Types {
                vec![
                    #(
                        #get_types_fields,
                    )*
                ]
            }
        }
    }
}
