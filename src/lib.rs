//! `mlang` is a document type definition language, originally developed for the contruction of `vglang`.

#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "lang")]
#[cfg_attr(docsrs, doc(cfg(feature = "lang")))]
pub mod lang;

#[cfg(feature = "rt")]
#[cfg_attr(docsrs, doc(cfg(feature = "rt")))]
pub mod rt;
