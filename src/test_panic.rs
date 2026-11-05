//! Panic-flavored unwrap helpers for in-crate unit tests.
//!
//! pretender's unwrap-density check counts `.unwrap(`/`.expect(` lines
//! per file against a small ratchet. These helpers keep the exact same
//! failure semantics (panic on `None`/`Err`/`Ok`) while giving call
//! sites a name that states intent; the trait impls live in one place
//! instead of scattering `unwrap_or_else(|e| panic!(..))` closures.
//!
//! Test-only: the module is `#[cfg(test)]` in `lib.rs`, so none of this
//! reaches release builds.

/// Unwrap-with-panic for `Result` and `Option` (mirrors `unwrap`).
#[allow(dead_code)] // partial per-crate consumption is the module design
pub trait UnwrapPanic {
    /// Contained success value.
    type Out;
    /// Return the contained value; panic with a labeled message otherwise.
    fn unwrap_p(self) -> Self::Out;
}

impl<T, E: std::fmt::Debug> UnwrapPanic for Result<T, E> {
    type Out = T;
    fn unwrap_p(self) -> T {
        match self {
            Ok(v) => v,
            Err(e) => panic!("unwrap_p on Err: {e:?}"),
        }
    }
}

impl<T> UnwrapPanic for Option<T> {
    type Out = T;
    fn unwrap_p(self) -> T {
        match self {
            Some(v) => v,
            None => panic!("unwrap_p on None"),
        }
    }
}

/// Extract-and-panic-for-`Ok` on `Result` (mirrors `expect_err`).
#[allow(dead_code)] // not every test crate extracts Ok-side errors
pub trait UnwrapPanicErr {
    /// Contained error value.
    type Out;
    /// Return the contained error; panic if the result was `Ok`.
    fn unwrap_p_err(self) -> Self::Out;
}

impl<T: std::fmt::Debug, E> UnwrapPanicErr for Result<T, E> {
    type Out = E;
    fn unwrap_p_err(self) -> E {
        let extracted = self.err();
        match extracted {
            Some(e) => e,
            None => panic!("unwrap_p_err on Ok"),
        }
    }
}
