//! The settings layers (ADM-04): the principal's configuration, the
//! repository configuration and managed policy, over the defaults of
//! §9.6.
//!
//! - [`key`]: the settings keys, what each takes, what a repository
//!   configuration may do with it (SEC-11) and which of its changes
//!   await acceptance.
//! - [`layer`]: one layer's TOML, validated strictly.
//! - [`resolve`]: the layers combined, managed policy over all.
//! - [`source`]: where each layer is read from, and the checks reading
//!   managed policy makes.
//! - [`value`]: a setting's value.
//!
//! Nothing here quotes a layer's text: a repository configuration's
//! author controls it (T5).

pub mod key;
pub mod layer;
pub mod resolve;
pub mod source;
pub mod value;

pub use resolve::{IgnoredKey, Layers, Reason, Resolved, Settings, SettingsLayer, resolve};
pub use source::{LayerFailure, Loaded, Sources, load};
pub use value::Value;
