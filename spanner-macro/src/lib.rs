use proc_macro::{TokenStream};
use syn::{ItemStruct, parse_macro_input};
use syn::ext::IdentExt;
use quote::quote;

#[proc_macro_derive(Spanner)]
pub fn convert_struct(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemStruct);
    let struct_name = item.ident;

    let mut to_kinds_fields = Vec::with_capacity(item.fields.len());
    let mut get_types_fields = Vec::with_capacity(item.fields.len());
    for field in &item.fields {
        let field_var = field.ident.as_ref().unwrap();
        let field_name = field_var.unraw().to_string();
        let ty = &field.ty;
        to_kinds_fields.push(quote! {
            (stringify!(#field_name), self.#field_var.to_kind())
        });
        get_types_fields.push(quote! {
            (stringify!(#field_name), #ty::get_type())
        });
    }

    let gen = quote! {
        use google_cloud_spanner::statement::{ToStruct, ToKind, Kinds, Types};

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
    };
    gen.into()
}