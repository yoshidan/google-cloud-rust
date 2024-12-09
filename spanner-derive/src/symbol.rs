use std::fmt;
use std::fmt::Display;

use syn::{Ident, Path};

#[derive(Copy, Clone)]
pub(crate) struct Symbol(&'static str);

pub(crate) const COMMIT_TIMESTAMP: Symbol = Symbol("commitTimestamp");
pub(crate) const COLUMN_NAME: Symbol = Symbol("name");
pub(crate) const COLUMN: Symbol = Symbol("spanner");

impl PartialEq<Symbol> for Ident {
    fn eq(&self, word: &Symbol) -> bool {
        self == word.0
    }
}

impl PartialEq<Symbol> for &Ident {
    fn eq(&self, word: &Symbol) -> bool {
        *self == word.0
    }
}

impl PartialEq<Symbol> for Path {
    fn eq(&self, word: &Symbol) -> bool {
        self.is_ident(word.0)
    }
}

impl PartialEq<Symbol> for &Path {
    fn eq(&self, word: &Symbol) -> bool {
        self.is_ident(word.0)
    }
}

impl Display for Symbol {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(self.0)
    }
}
