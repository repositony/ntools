use crate::f;

/// Extends Option for easy display formatting
pub trait OptionExt {
    /// Better option outputs
    ///
    /// Generic over anything that implements `Display`, this will either be the
    /// value contained within `Some()` or "none" for the `None` variant.
    ///
    /// For example:
    ///
    /// ```rust
    /// # use ntools_utils::OptionExt;
    /// let x: Option<u32> = Some(2);
    /// assert_eq!(x.display(), "2");
    ///
    /// let x: Option<u32> = None;
    /// assert_eq!(x.display(), "none");
    /// ```
    fn display(&self) -> String;
}

impl<T: std::fmt::Display> OptionExt for Option<T> {
    fn display(&self) -> String {
        match self {
            Some(value) => f!("{value}"),
            None => "none".to_string(),
        }
    }
}
