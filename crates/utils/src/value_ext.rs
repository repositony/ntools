use crate::f;

/// Extends primitives with more specific formatting options
pub trait ValueExt {
    /// Better scientific number formatting
    ///
    /// The default is not very consistent for scientific in particular, so this
    /// allows easy definition.
    ///
    /// Works for anything that can be represented as scientific using the
    /// `LowerExp` trait, which is pretty much every numerical primitive.
    ///
    /// ```rust
    /// # use ntools_utils::ValueExt;
    /// let number = -1.0;
    /// assert_eq!(number.sci(5, 2), "-1.00000e+00".to_string());
    /// assert_eq!((1.0).sci(5, 2), "1.00000e+00".to_string());
    /// ```
    fn sci(&self, precision: usize, exp_pad: usize) -> String;
}

impl<T: std::fmt::LowerExp> ValueExt for T {
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
