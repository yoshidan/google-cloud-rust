use proc_macro::{TokenStream};
use convert_case::{Case, Casing};
use syn::{Error, Ident, ItemStruct, parse_macro_input};
use syn::ext::IdentExt;
use quote::{quote, ToTokens};
use syn::Lit::Str;
use syn::Meta::{List, NameValue, Path};
use syn::NestedMeta::Meta;
use syn::spanned::Spanned;
use crate::column::Column;
use crate::symbol::{COLUMN, COLUMN_NAME, COMMIT_TIMESTAMP};

pub(crate) fn generate_table_methods(item: ItemStruct) -> impl ToTokens {
    let struct_name = item.ident;

    let mut to_kinds_fields = Vec::with_capacity(item.fields.len());
    let mut get_types_fields = Vec::with_capacity(item.fields.len());
    for field in &item.fields {
        let field_var = field.ident.as_ref().unwrap();
        let column = Column::from(field);
        let column_name = column.name();
        let ty = &field.ty;
        let mut get_field_type  =  quote! { #ty };
        let mut to_kind_field_type = quote! { self.#field_var };
        if column.commit_timestamp {
            get_field_type = quote! { CommitTimestamp };
            to_kind_field_type = quote! { CommitTimestamp::new() };
        }
        to_kinds_fields.push(quote! {
            (stringify!(#column_name), #to_kind_field_type.to_kind())
        });
        get_types_fields.push(quote! {
            (stringify!(#column_name), #get_field_type::get_type())
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
