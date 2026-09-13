//! Replacing a file without ever emptying the one already there.
//!
//! Every document this app writes — the save, the Markdown export, the settings
//! file — used `fs::write`, which is `File::create` (`O_TRUNC`) plus
//! `write_all`. That empties the existing file *before* the new bytes land, so a
//! failure in between (a full disk, an unmounted drive, a killed process) leaves
//! the user with nothing at all. Plain Save writes straight to the current path
//! with no dialog, dozens of times a session, and the app keeps no second copy
//! anywhere — so the truncating write is one interruption away from destroying a
//! character outright, which is the data-loss class `CLAUDE.md` rates highest.
//!
//! The replacement is the standard write-temp-then-rename: write the whole
//! document to a scratch file, flush it to disk with [`std::fs::File::sync_all`]
//! so the bytes are really there, then [`std::fs::rename`] it over the target —
//! an atomic replace on every platform this app ships to, so a reader sees
//! either the complete old file or the complete new one and never a half-written
//! or empty one.
//!
//! **The scratch file goes in the target's own directory, and that is
//! load-bearing.** A rename is only atomic within one filesystem; a scratch file
//! in the OS temp directory would very often be on another one, where
//! `fs::rename` fails outright (`EXDEV`) or degrades to a copy — which is
//! exactly the non-atomic overwrite this module exists to avoid.

use std::ffi::OsString;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::AppError;

/// Distinguishes concurrent writes within one process; the process id
/// distinguishes them across processes. Two saves racing for one path is not a
/// flow this app has, but a scratch name two writers could both pick would turn
/// that into a corrupt file rather than a lost race, and a counter is cheaper
/// than reasoning about it.
static SCRATCH_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Writes `contents` to `path`, replacing whatever is there as one step.
///
/// On any failure the target keeps the bytes it had, and no scratch file is left
/// behind. See the module header for why the scratch file must share the
/// target's directory.
pub fn write_file_atomically(path: &Path, contents: &str) -> Result<(), AppError> {
    let scratch = scratch_path(path)?;
    match write_then_rename(&scratch, path, contents) {
        Ok(()) => Ok(()),
        Err(error) => {
            // Best effort: the write already failed, and a scratch file that
            // cannot be removed must not replace the error that explains why.
            let _ = fs::remove_file(&scratch);
            Err(io_error_at(path, &error))
        }
    }
}

fn write_then_rename(scratch: &Path, target: &Path, contents: &str) -> std::io::Result<()> {
    let mut file = File::create(scratch)?;
    file.write_all(contents.as_bytes())?;
    // Flush to the device before the rename: a rename that beats the data to
    // disk would publish a file whose contents are not there yet.
    file.sync_all()?;
    drop(file);
    // A fresh file carries the process umask, not the mode the user gave the
    // document it replaces — so carry the existing mode over when there is one.
    if let Ok(existing) = fs::metadata(target) {
        let _ = fs::set_permissions(scratch, existing.permissions());
    }
    fs::rename(scratch, target)
}

/// A scratch name beside `target`, hidden (leading dot) so a crash between the
/// write and the rename leaves nothing obtrusive in the user's save folder.
///
/// Built from `OsString` rather than `String` because a path component on Linux
/// is an arbitrary byte string, and going through `to_string_lossy` here would
/// invent a name unrelated to the file being written.
fn scratch_path(target: &Path) -> Result<PathBuf, AppError> {
    let file_name = target.file_name().ok_or_else(|| AppError::Io {
        message: format!("{} names no file to write", target.display()),
    })?;
    let directory = match target.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        // A bare file name is relative to the working directory, which is also
        // where the rename has to happen.
        _ => Path::new("."),
    };
    let mut name = OsString::from(".");
    name.push(file_name);
    name.push(format!(
        ".tmp-{}-{}",
        std::process::id(),
        SCRATCH_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    Ok(directory.join(name))
}

/// Names the file the caller asked to write, because the underlying failure
/// ("No such file or directory") may be about the scratch file and says nothing
/// about which document was being saved.
fn io_error_at(target: &Path, error: &std::io::Error) -> AppError {
    AppError::Io {
        message: format!("{}: {error}", target.display()),
    }
}
