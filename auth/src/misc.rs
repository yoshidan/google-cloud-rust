pub(crate) const EMPTY: &str = "";

pub(crate) trait UnwrapOrEmpty<T> {
    fn unwrap_or_empty(&self) -> T;
}

impl UnwrapOrEmpty<String> for Option<String> {
    fn unwrap_or_empty(&self) -> String {
        match self {
            Some(s) => s.to_string(),
            None => EMPTY.to_string(),
        }
    }
}
