//! DPAPI for Windows' key store (SEC-10, the plan's ruling 2): a key's
//! bytes protected for the running OS user before they reach a file, and
//! unprotected when read back.
//!
//! The workspace forbids unsafe code; this crate is the one exception,
//! and only its [`shim`] module allows it. The shim calls DPAPI through
//! the [`Api`] trait, which Windows implements through `windows-sys` and
//! a test fakes, so Miri checks every pointer the shim handles on any OS
//! (ENG-09).

#[allow(unsafe_code)]
mod shim;

#[cfg(windows)]
pub use shim::Windows;
pub use shim::{Api, Blob, Error, protect, unprotect};
