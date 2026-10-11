//! Last line of defence: a bug that panics becomes an error, not a crash.
//!
//! This doesn't replace `Result`. Code returns errors; the guard only exists so that a missed case
//! rolls the command back instead of taking the app (and unsaved work) down. Wasm aborts on panic,
//! so on the web the no-panic rule is the only protection.

use std::panic::{AssertUnwindSafe, catch_unwind};

pub fn catch_panic<T>(f: impl FnOnce() -> T) -> Result<T, String> {
    catch_unwind(AssertUnwindSafe(f)).map_err(|payload| {
        let msg = if let Some(s) = payload.downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic".to_string()
        };
        log::error!("recovered from a panic: {msg}");
        format!("internal error: {msg}")
    })
}
