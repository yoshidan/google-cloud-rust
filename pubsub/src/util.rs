pub(crate) trait ToUsize{
    fn to_usize(&self) -> usize;
}

impl ToUsize for &str {
    fn to_usize(&self) -> usize {
        self.as_bytes().iter().sum()
    }
}