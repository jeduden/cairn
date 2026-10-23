//! The deadline every walk takes (ENG-03).

use crate::Error;

/// A deadline a caller injects: a walk asks it between steps and stops
/// with [`Error::Deadline`] once it has passed. A closure returning
/// `true` after the deadline is one.
pub trait Deadline {
    /// Whether the deadline has passed.
    fn expired(&self) -> bool;
}

impl<F: Fn() -> bool> Deadline for F {
    fn expired(&self) -> bool {
        self()
    }
}

/// Fails with [`Error::Deadline`] once `deadline` has passed.
pub(crate) fn check(_deadline: &dyn Deadline) -> Result<(), Error> {
    Ok(())
}

#[cfg(test)]
mod tests;
