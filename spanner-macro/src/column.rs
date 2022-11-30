use crate::symbol::{COLUMN, COLUMN_NAME, COMMIT_TIMESTAMP};
use convert_case::{Case, Casing};
use syn::ext::IdentExt;
use syn::spanned::Spanned;
use syn::Lit::Str;
use syn::Meta::{List, NameValue, Path};
use syn::NestedMeta::Meta;
use syn::{Error, Field};

pub(crate) struct Column<'a> {
    field: &'a Field,
    pub column_name: Option<String>,
    pub commit_timestamp: bool,
}

impl<'a> Column<'a> {
    pub(crate) fn name(&self) -> String {
        match &self.column_name {
            Some(v) => v.to_string(),
            None => {
                let field_var = self.field.ident.as_ref().unwrap();
                field_var.unraw().to_string().to_case(Case::Title)
            }
        }
    }
}

impl<'a> From<&'a Field> for Column<'a> {
    /// Extract out the `#[column(...)]` attributes from a struct field.
    fn from(field: &'a Field) -> Self {
        let mut commit_timestamp = false;
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
            field,
            commit_timestamp,
            column_name,
        }
    }
}

fn get_meta_items(attr: &syn::Attribute) -> Result<Vec<syn::NestedMeta>, Error> {
    if attr.path != COLUMN {
        return Ok(Vec::new());
    }

    match attr.parse_meta()? {
        List(meta) => Ok(meta.nested.into_iter().collect()),
        _ => Err(Error::new(attr.span(), "expected [column(...)]")),
    }
}
