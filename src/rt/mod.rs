//! Rust runtime library for generation codes.

pub mod opcode;

#[cfg(feature = "rt_serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "rt_serde")))]
pub mod serde;
