//! Common utility for extended `std` types
//!
//! These are left public for convenience.
//!
//! For example, convenient functions that capitalise a string or extensions to
//! vector functionality are useful everywhere.

// Alias for the format! macro
pub use std::format as f;

// Modules
mod error;
mod option_ext;
mod slice_ext;
mod sort_ext;
mod string_ext;
mod value_ext;

// Flatten
pub use error::Error;
pub use option_ext::OptionExt;
pub use slice_ext::SliceExt;
pub use sort_ext::SortExt;
pub use string_ext::StringExt;
pub use value_ext::ValueExt;
