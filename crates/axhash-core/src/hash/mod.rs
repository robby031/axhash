pub mod api;
pub mod build;
pub mod core;
// Implementation detail: contains only `impl Hasher for AxHasher`.
// Nothing here is part of the public API.
pub(crate) mod hasher_impl;

pub use crate::hash::core::AxHasher;
