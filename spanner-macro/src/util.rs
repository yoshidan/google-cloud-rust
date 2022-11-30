use proc_macro::{TokenStream};
use quote::{quote, ToTokens};

pub(crate) fn wrap_in_dummy_mod(item: impl ToTokens) -> TokenStream {
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