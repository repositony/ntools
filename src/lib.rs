//! A semi-modular toolkit of fast and reliable libraries for neutronics
//! analysis
//!
#![doc = include_str!("../readme.md")]
#![deny(missing_docs, missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

// Flatten the default crates
#[doc(inline)]
pub use ntools_utils as utils;

// Re-exports of crates that are behind feature flags
#[cfg(feature = "fispact")]
#[cfg_attr(docsrs, doc(cfg(feature = "fispact")))]
#[doc(inline)]
pub use ntools_fispact as fispact;

#[cfg(feature = "iaea")]
#[cfg_attr(docsrs, doc(cfg(feature = "iaea")))]
#[doc(inline)]
pub use ntools_iaea as iaea;

#[cfg(feature = "mesh")]
#[cfg_attr(docsrs, doc(cfg(feature = "mesh")))]
#[doc(inline)]
pub use ntools_mesh as mesh;

#[cfg(feature = "mctal")]
#[cfg_attr(docsrs, doc(cfg(feature = "mctal")))]
#[doc(inline)]
pub use ntools_mctal as mctal;

#[cfg(feature = "posvol")]
#[cfg_attr(docsrs, doc(cfg(feature = "posvol")))]
#[doc(inline)]
pub use ntools_posvol as posvol;

#[cfg(feature = "weights")]
#[cfg_attr(docsrs, doc(cfg(feature = "weights")))]
#[doc(inline)]
pub use ntools_weights as weights;

#[cfg(feature = "wwgen")]
#[cfg_attr(docsrs, doc(cfg(feature = "wwgen")))]
#[doc(inline)]
pub use ntools_wwgen as wwgen;
