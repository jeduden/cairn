//! Windows' key store (the plan's ruling 2): a key's bytes protected by
//! DPAPI for the running OS user before they reach a file, through the
//! `dpapi` crate, the one crate that holds `unsafe` code.

use super::Protect;

/// DPAPI, scoped to the running OS user.
#[derive(Debug, Clone, Copy)]
pub struct Dpapi;

impl Protect for Dpapi {
    fn protect(&self, plain: &[u8]) -> Result<Vec<u8>, String> {
        dpapi::protect(&dpapi::Windows, plain).map_err(|e| e.to_string())
    }

    fn unprotect(&self, blob: &[u8]) -> Result<Vec<u8>, String> {
        dpapi::unprotect(&dpapi::Windows, blob).map_err(|e| e.to_string())
    }
}
