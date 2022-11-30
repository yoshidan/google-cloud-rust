use syn::Error;
use syn::Lit::Str;
use syn::Meta::{List, NameValue, Path};
use syn::NestedMeta::Meta;
use syn::spanned::Spanned;
use crate::symbol::{COLUMN, COLUMN_NAME, COMMIT_TIMESTAMP};

pub(crate) struct Column {
    pub column_name: Option<String>,
    pub commit_timestamp : bool
}

impl Column {
    /// Extract out the `#[column(...)]` attributes from a struct field.
    pub(crate) fn from_ast(field: &syn::Field) -> Self {
        let mut commit_timestamp= false;
        let mut column_name = None;
        for meta_item in field.attrs.iter().flat_map(|attr| get_meta_items(attr).unwrap()) {
            match &meta_item {
                // Parse `#[column(name = "foo")]`
                Meta(NameValue(m)) if m.path == COLUMN_NAME => {
                    if let Str(s) = &m.lit {
                        column_name = Some(s.value());
                    }
                }
                // Parse `#[column(commitTimestamp)]`
                Meta(Path(word)) if word == COMMIT_TIMESTAMP => {
                    commit_timestamp = true;
                }
                _ => {}
            }
        }

        Self {
            commit_timestamp,
            column_name
        }
    }
}


fn get_meta_items(attr: &syn::Attribute) -> Result<Vec<syn::NestedMeta>, Error> {
    if attr.path != COLUMN {
        return Ok(Vec::new());
    }

    match attr.parse_meta()? {
        List(meta) => Ok(meta.nested.into_iter().collect()),
        _ => {
            Err(Error::new(attr.span(), "expected [column(...)]"))
        }
    }
}
