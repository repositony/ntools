/// Extends string types with useful functions
pub trait StringExt {
    /// Capilalises the first letter in a string
    ///
    /// ```rust
    /// # use ntools_utils::StringExt;
    /// assert_eq!("test string".capitalise(), "Test string".to_string());
    /// ```
    fn capitalise(&self) -> String;
}

impl<T: AsRef<str>> StringExt for T {
    fn capitalise(&self) -> String {
        let mut c = self.as_ref().chars();
        match c.next() {
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            None => String::new(),
        }
    }
}
