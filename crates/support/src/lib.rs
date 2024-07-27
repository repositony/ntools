//! Common utility for extended `std` types
//!
//! These are left public for convenience.
//!
//! For example, capitalising a string or using prettier formatting for
//! scientific numbers are useful everywhere.

// standard library
use std::fmt::{Display, LowerExp};

// Alias for the format! macro
pub use std::format as f;

/// Extends primitives with more specific formatting options
pub trait FloatExt {
    /// Better scientific number formatting
    ///
    /// The default is not very consistent for scientific in particular, so this
    /// allows easy definition.
    ///
    /// Works for anything that can be represented as scientific using the
    /// `LowerExp` trait, which is pretty much every numerical primitive.
    ///
    /// ```rust
    /// # use ntools_support::FloatExt;
    /// let number = -1.0;
    /// assert_eq!(number.sci(5, 2), "-1.00000e+00".to_string());
    /// assert_eq!((1.0).sci(5, 2), "1.00000e+00".to_string());
    /// ```
    fn sci(&self, precision: usize, exp_pad: usize) -> String;
}

impl<T: LowerExp> FloatExt for T {
    fn sci(&self, precision: usize, exp_pad: usize) -> String {
        let mut num = f!("{:.precision$e}", &self, precision = precision);
        // Safe to `unwrap` as `num` is guaranteed to contain `'e'`
        let exp = num.split_off(num.find('e').unwrap());
        // Make sure the exponent is signed
        let (sign, exp) = match exp.strip_prefix("e-") {
            Some(exp) => ('-', exp),
            None => ('+', &exp[1..]),
        };
        // Pad the exponent with zeros if needed and put it back on the number
        num.push_str(&f!("e{}{:0>pad$}", sign, exp, pad = exp_pad));
        num
    }
}

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
    /// # use ntools_support::OptionExt;
    /// let x: Option<u32> = Some(2);
    /// assert_eq!(x.display(), "2");
    ///
    /// let x: Option<u32> = None;
    /// assert_eq!(x.display(), "none");
    /// ```
    fn display(&self) -> String;
}

impl<T: Display> OptionExt for Option<T> {
    fn display(&self) -> String {
        match self {
            Some(value) => f!("{value}"),
            None => "none".to_string(),
        }
    }
}

/// Extends Option for easy display formatting
pub trait StringExt {
    /// Capilalises the first letter in a string
    ///
    /// ```rust
    /// # use ntools_support::StringExt;
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
