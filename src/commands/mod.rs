// Command handlers, grouped by area (split from main.rs — specodelic-g17).
pub(crate) mod corpus;
pub(crate) mod manage;
pub(crate) mod model_check;

use genesis::guide::Output;

/// specodelic-4v1 (F1): a failing run must never ride a success-shaped
/// envelope — flip the envelope kind to error while keeping the payload,
/// so consumers gating on `.ok` see the failure and still read the
/// per-file detail (specs/errors.md `envelope_error_kind` +
/// `exit_code_mapping`). The message leads the warnings channel (the
/// genesis failure convention: failure messages are always visible, and
/// in JSON mode they also print to stderr).
pub(crate) fn as_failure<T: std::fmt::Debug>(
    out: Output<T>,
    message: impl Into<String>,
) -> Output<T> {
    let mut out = out;
    out.is_error = true;
    out.warnings.insert(0, message.into());
    // Failure outputs stay visible at Quiet, matching Output::failure.
    out.verbosity = 0;
    out
}
