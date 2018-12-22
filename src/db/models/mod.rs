// TODO: remove this once diesel 1.4 is released
#![allow(proc_macro_derive_resolution_fallback)]

pub use self::configs::*;
pub use self::schema_versions::*;
pub use self::schemas::*;
pub use self::subjects::*;

pub mod schema;

mod configs;
mod schema_versions;
mod schemas;
mod subjects;
