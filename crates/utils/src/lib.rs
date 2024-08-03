//! Common utility for extended `std` types
//!
//! These are left public for convenience.
//!
//! For example, capitalising a string or using prettier formatting for
//! scientific numbers are useful everywhere.

// Alias for the format! macro
pub use std::format as f;

// Modules
mod option_ext;
mod sort_ext;
mod string_ext;
mod value_ext;

// Flatten
pub use option_ext::OptionExt;
pub use sort_ext::SortExt;
pub use string_ext::StringExt;
pub use value_ext::ValueExt;
