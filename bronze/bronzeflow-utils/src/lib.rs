//! # bronzeflow-utils: Some common util for bronzeflow
//!
//! This is a temporary internal dependency for bronzeflow

mod log;
pub mod prelude;

pub type Result<T> = anyhow::Result<T>;
pub type BronzeError = anyhow::Error;

pub use anyhow::anyhow as ayn_error;
