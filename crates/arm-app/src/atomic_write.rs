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
//!
//! # What "the target" means, and what the replace costs
//!
//! Replacing a file by rename installs a *new inode* where the old one was,
//! while the truncating write kept writing into the old one. Three consequences
//! follow, and each is decided here rather than left to whatever `rename(2)`
//! happens to do (Gerda #2, Klaus K1/K2, round 2):
//!
//! - **A symlinked target is followed, not replaced.** `rename(2)` does not
//!   follow a symlink in its final component: it unlinks the link and puts the
//!   new file where the link was. Keeping a save as a link into a synced folder
//!   is an ordinary thing to do on a desktop with no sync of its own, and that
//!   behaviour would silently detach it — the save reports success, the dirty
//!   flag clears, and the synced copy is never written again. So the final
//!   component is resolved first ([`resolved_target`]) and the whole operation,
//!   scratch file included, happens beside the *real* file. **A link that points
//!   at a file which does not exist yet is followed just the same** — preparing
//!   the link before the first save is the ordinary way to set this up, and
//!   giving up on resolving it is how the save came to destroy the link it could
//!   not follow (Gerda #2, round 3).
//! - **A write-protected target is refused, loudly.** `rename(2)` needs write
//!   permission on the containing directory and none at all on the destination,
//!   so a `chmod 444` character the user marked "finished" would be replaced
//!   without complaint — where the truncating write failed with `EACCES`, which
//!   is the entire point of setting the mode. [`refuse_write_protected`]
//!   restores that refusal.
//! - **Everything else the old inode carried is still lost** — POSIX ACLs and
//!   extended attributes (macOS Finder tags live in one). Only the mode bits are
//!   carried over, which is what [`std::fs::Permissions`] models; copying the
//!   rest is per-platform plumbing for metadata this app never reads. The
//!   trade-off is deliberate and recorded here rather than implemented: the
//!   document's *contents* are always correct, and that is worth more.
//!
//! Cleanup is likewise not left to chance. A process killed between the write
//! and the rename strands a scratch file that is a **complete copy of the
//! document**, and the window spans `sync_all`, which is the slow part — so the
//! likeliest moment to be interrupted is the moment the copy is fully written.
//! The leading dot hides it on Unix but not in Windows Explorer, a declared
//! target, so every successful write also sweeps the strays its own target left
//! behind ([`sweep_stranded_scratch_files`]), which bounds them at one.

use std::ffi::{OsStr, OsString};
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
/// target's directory, what happens when `path` is a symlink, and why a
/// write-protected target is refused.
///
/// Errors always name `path` — the document the caller asked to write — because
/// the underlying failure may be about the scratch file and says nothing about
/// which save it was for.
pub fn write_file_atomically(path: &Path, contents: &str) -> Result<(), AppError> {
    let target = resolved_target(path)?;
    refuse_write_protected(path, &target)?;
    let scratch = scratch_path(&target)?;
    match write_then_rename(&scratch, &target, contents) {
        Ok(()) => Ok(()),
        Err(error) => {
            // Best effort: the write already failed, and a scratch file that
            // cannot be removed must not replace the error that explains why.
            let _ = fs::remove_file(&scratch);
            Err(io_error_at(path, &error))
        }
    }
}

/// How many links deep the walk below follows before calling the path a cycle.
/// Chains this long are already pathological; the bound exists so a link loop
/// ends the walk rather than spinning it.
const MAX_SYMLINK_HOPS: u8 = 16;

/// The file the write must actually land on: `path` itself, or — when its final
/// component is a symlink — the path at the end of that chain of links,
/// **whether or not anything exists there**.
///
/// Only the final component is resolved, because that is the only one
/// [`std::fs::rename`] treats differently from `open(2)`: a symlinked *directory*
/// along the way is followed identically by both, and the scratch file travels
/// the same path as the target, so the rename still happens within one
/// filesystem. Resolving the final link, by contrast, is what keeps a save
/// pointed into a synced folder from being replaced by a regular file.
///
/// A **dangling** link is resolved just the same, and that is the case
/// [`fs::canonicalize`] cannot serve: it requires every component to exist, so
/// the link a player prepares before their first save — the ordinary way to route
/// a document into a synced folder — errored, and falling back to the link's own
/// path let `rename` unlink it and install a regular file in its place (Gerda #2,
/// round 3). `File::create` carries no `O_NOFOLLOW`, so the truncating write this
/// module replaced resolved that link and created the file at its *target*,
/// leaving the link intact; following the chain here is what preserves that.
///
/// A link *cycle* resolves to nothing at all, and the write is refused — the
/// `ELOOP` the truncating write got from `open(2)`. Guessing a path here would
/// mean dropping a regular file over a link and reporting success, which is the
/// same silent detachment by another route.
fn resolved_target(path: &Path) -> Result<PathBuf, AppError> {
    let mut current = path.to_path_buf();
    for _ in 0..MAX_SYMLINK_HOPS {
        // Anything that cannot be stat'ed is not a link this can follow: a
        // missing file is the ordinary first save, and a stat that fails for
        // any other reason fails again, with its own error, at the write.
        let Ok(metadata) = fs::symlink_metadata(&current) else {
            return Ok(current);
        };
        if !metadata.file_type().is_symlink() {
            return Ok(current);
        }
        let link = fs::read_link(&current).map_err(|error| io_error_at(path, &error))?;
        current = match current.parent() {
            // A relative link resolves against the directory holding the link,
            // never the process's working directory.
            Some(directory) if link.is_relative() => directory.join(link),
            _ => link,
        };
    }
    // `ErrorKind::FilesystemLoop` is the name for this, and is still unstable —
    // the wording is `open(2)`'s own, which is what the caller would have seen
    // before this module existed.
    Err(io_error_at(
        path,
        &std::io::Error::other("too many levels of symbolic links"),
    ))
}

/// Refuses to replace a target the user has write-protected, restoring the
/// refusal the truncating write got for free from `File::create` (Klaus K1).
///
/// A missing target is not protected — that is an ordinary first save.
/// [`std::fs::Permissions::readonly`] is the portable spelling of the marker the
/// user actually sets (`chmod 444`, or the Windows read-only attribute); it is
/// not a full access check, so a target that is unwritable for some other reason
/// — owned by another user, on a read-only mount — still fails later, at
/// `fs::rename`, which is where it failed before this check existed too.
fn refuse_write_protected(requested: &Path, target: &Path) -> Result<(), AppError> {
    let Ok(metadata) = fs::metadata(target) else {
        return Ok(());
    };
    if !metadata.permissions().readonly() {
        return Ok(());
    }
    Err(io_error_at(
        requested,
        &std::io::Error::from(std::io::ErrorKind::PermissionDenied),
    ))
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
    sweep_stranded_scratch_files(target, scratch);
    fs::rename(scratch, target)
}

/// Removes scratch files an earlier interrupted write left beside `target`.
///
/// Best effort throughout: an orphan that cannot be read or removed is a tidiness
/// problem, and it must never fail the save it is riding along with.
///
/// Scoped to `target`'s own scratch names — which is the whole safety argument
/// for sweeping at all. The prefix carries the target's full file name, so
/// another document's scratch file, and any ordinary file of the user's that
/// merely shares the stem, are outside it. The one thing the scope cannot
/// exclude is a *concurrent* write of the same document from another process;
/// that would cost the other writer its scratch file and fail its save safely,
/// and this app has no flow that saves one path twice at once.
fn sweep_stranded_scratch_files(target: &Path, keep: &Path) {
    let Some(file_name) = target.file_name() else {
        return;
    };
    let Some(directory) = keep.parent() else {
        return;
    };
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    let prefix = scratch_prefix(file_name);
    for entry in entries.flatten() {
        if is_scratch_name(&entry.file_name(), &prefix) && entry.path() != keep {
            let _ = fs::remove_file(entry.path());
        }
    }
}

/// Whether `name` is one of the scratch names [`scratch_path`] builds for a
/// target whose scratch prefix is `prefix`.
///
/// Compared on the encoded bytes rather than through `to_string_lossy`, for the
/// same reason the names are *built* from `OsString`: a path component on Linux
/// is an arbitrary byte string, and two different names can share one lossy
/// spelling — which here would mean deleting a file that is not ours.
fn is_scratch_name(name: &OsStr, prefix: &OsStr) -> bool {
    name.as_encoded_bytes()
        .starts_with(prefix.as_encoded_bytes())
}

/// The shared leading part of every scratch name for one target: a leading dot,
/// the target's own file name, and the `.tmp-` marker the unique suffix follows.
fn scratch_prefix(file_name: &OsStr) -> OsString {
    let mut prefix = OsString::from(".");
    prefix.push(file_name);
    prefix.push(".tmp-");
    prefix
}

/// A scratch name beside `target`, hidden by the leading dot **on Unix** so a
/// crash between the write and the rename leaves nothing obtrusive in the user's
/// save folder. Windows Explorer does not honour that convention, which is why
/// [`sweep_stranded_scratch_files`] — not the dot — is what actually bounds the
/// strays on every platform.
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
    let mut name = scratch_prefix(file_name);
    name.push(format!(
        "{}-{}",
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
