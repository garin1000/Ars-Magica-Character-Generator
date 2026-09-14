//! What replacing a file by rename costs, and what it must not cost.
//!
//! `atomic_write` exists so a save can never empty the user's only copy, and it
//! buys that with write-temp-then-rename. Round 2 found three things that
//! trade-off silently took away, all three of them about the *file* rather than
//! its contents — a symlinked save stopped being followed (Gerda #2), a
//! write-protected save stopped being protected (Klaus K1), and an interrupted
//! save left a full copy of the document behind forever (Klaus K2). Nothing
//! anywhere asserted on any of it: before this file, no test in the repository
//! touched `permissions`, `readonly` or a symlinked target at all.
//!
//! These are integration tests because they are about the filesystem, not about
//! the module's internals: every one of them sets up a real file, calls the same
//! public entry point `save_entity_to_path` and `write_settings` use, and then
//! looks at the directory the way the user would.

use std::fs;
use std::path::Path;

use arm_app::atomic_write::write_file_atomically;

/// Gerda #2: `rename(2)` does not follow a symlink in its final component — it
/// unlinks the link and drops a regular file in its place. `fs::write`, which
/// the atomic write replaced, opened the target with `O_TRUNC` and therefore
/// wrote *through* the link.
///
/// The failure this pins is silent and permanent: a player keeps
/// `~/saves/magus.armc` as a link into a synced folder, and after the first
/// Ctrl+S the link is gone, the synced copy is frozen at the pre-session
/// version, and nothing tells them — the save reported success and cleared the
/// dirty flag.
#[cfg(unix)]
#[test]
fn a_save_through_a_symlink_updates_the_file_the_link_points_at() {
    let tmp = tempfile::tempdir().unwrap();
    let real = tmp.path().join("synced").join("magus.armc");
    fs::create_dir_all(real.parent().unwrap()).unwrap();
    fs::write(&real, "old").unwrap();

    let link = tmp.path().join("magus.armc");
    std::os::unix::fs::symlink(&real, &link).unwrap();

    write_file_atomically(&link, "new").unwrap();

    assert!(
        fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink(),
        "the save must write through the link, not replace it with a regular \
         file — replacing it silently detaches the user's save from wherever \
         they pointed it"
    );
    assert_eq!(
        fs::read_to_string(&real).unwrap(),
        "new",
        "the file the link points at is the one that must receive the bytes"
    );
}

/// The same guarantee still has to hold through the link: the file the link
/// points at must be *replaced*, not truncated, or following the link would have
/// undone the whole point of the module. A hard link to the old bytes is the
/// witness, exactly as in `a_re_save_replaces_the_previous_file_instead_of_truncating_it`.
#[cfg(unix)]
#[test]
fn a_save_through_a_symlink_still_replaces_rather_than_truncates() {
    let tmp = tempfile::tempdir().unwrap();
    let real = tmp.path().join("magus.armc");
    fs::write(&real, "old").unwrap();
    let witness = tmp.path().join("previous-bytes");
    fs::hard_link(&real, &witness).unwrap();

    let link = tmp.path().join("link.armc");
    std::os::unix::fs::symlink(&real, &link).unwrap();

    write_file_atomically(&link, "new").unwrap();

    assert_eq!(
        fs::read_to_string(&witness).unwrap(),
        "old",
        "the previous bytes must survive: writing through the link must still \
         be a replace, never an in-place truncation"
    );
    assert_eq!(fs::read_to_string(&real).unwrap(), "new");
}

/// Gerda #2 (round 3): a link that points at a file which does not exist *yet* is
/// the ordinary way to prepare a save path — the player links
/// `~/chars/gerhard.armc` at a synced folder before the first save. The write must
/// create the file **at the link's target**, exactly as `fs::write` did
/// (`File::create` carries no `O_NOFOLLOW`, so `open(2)` resolves the final link
/// and creates the destination, leaving the link intact and now valid).
///
/// Round 2 resolved the link with `fs::canonicalize`, which errors on a link that
/// points nowhere, and fell back to the link's own path — so `fs::rename` unlinked
/// the link and installed a regular file where it had been. That is the same
/// silent detachment the follow-the-link fix exists to prevent, in the one case
/// the fix did not cover: the synced folder never receives the character, and
/// every later save keeps going to the wrong place with the app reporting success.
///
/// The old assertion (`read_to_string(&link) == "new"`) could not see any of this:
/// it holds both when the link is replaced by a regular file and when the target
/// is correctly created, because in the second case the link then resolves.
#[cfg(unix)]
#[test]
fn a_dangling_symlink_is_written_through_rather_than_replaced() {
    let tmp = tempfile::tempdir().unwrap();
    let link = tmp.path().join("magus.armc");
    let target = tmp.path().join("gone.armc");
    std::os::unix::fs::symlink(&target, &link).unwrap();

    write_file_atomically(&link, "new").unwrap();

    assert!(
        fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink(),
        "a dangling link must survive the save that fills it in — replacing it \
         with a regular file detaches the user's save from wherever they pointed \
         it, silently and permanently"
    );
    assert_eq!(
        fs::read_to_string(&target).unwrap(),
        "new",
        "the bytes must land where the link points, which is what the truncating \
         write did through O_CREAT"
    );
    assert!(no_scratch_files_beside(tmp.path()));
}

/// The same case one hop further out, and the reason a single `read_link` is not
/// enough: `magus.armc -> sync.armc -> gone.armc`, where only the last name is
/// missing. Stopping at the first hop would write over `sync.armc` — destroying an
/// intermediate link instead of the final one, which is the identical defect moved
/// along by one.
///
/// The links are **relative**, which is how a link inside a save folder is usually
/// written, so this also pins that a relative target is resolved against the
/// link's own directory rather than the process's working directory.
#[cfg(unix)]
#[test]
fn a_dangling_chain_of_symlinks_is_written_through_at_its_far_end() {
    let tmp = tempfile::tempdir().unwrap();
    let first = tmp.path().join("magus.armc");
    let second = tmp.path().join("sync.armc");
    let end = tmp.path().join("gone.armc");
    std::os::unix::fs::symlink("sync.armc", &first).unwrap();
    std::os::unix::fs::symlink("gone.armc", &second).unwrap();

    write_file_atomically(&first, "new").unwrap();

    assert!(
        fs::symlink_metadata(&first)
            .unwrap()
            .file_type()
            .is_symlink(),
        "the link the user named must survive"
    );
    assert!(
        fs::symlink_metadata(&second)
            .unwrap()
            .file_type()
            .is_symlink(),
        "an intermediate link must survive too — writing over it is the same \
         detachment one hop in"
    );
    assert_eq!(
        fs::read_to_string(&end).unwrap(),
        "new",
        "the bytes must land at the end of the chain"
    );
}

/// A link that points at itself (or round a cycle) resolves to nothing at all, and
/// there is no honest file to write. `fs::write` failed here with `ELOOP`, and so
/// must this: the one thing the save must NOT do is give up on resolving and drop
/// a regular file over the link, because that is indistinguishable from a
/// successful save to the user while their real destination is gone.
#[cfg(unix)]
#[test]
fn a_symlink_loop_fails_the_save_instead_of_replacing_the_link() {
    let tmp = tempfile::tempdir().unwrap();
    let first = tmp.path().join("magus.armc");
    let second = tmp.path().join("other.armc");
    std::os::unix::fs::symlink("other.armc", &first).unwrap();
    std::os::unix::fs::symlink("magus.armc", &second).unwrap();

    let result = write_file_atomically(&first, "new");

    assert!(
        result.is_err(),
        "a save that cannot resolve where the document goes must say so, not \
         guess"
    );
    assert!(
        fs::symlink_metadata(&first)
            .unwrap()
            .file_type()
            .is_symlink(),
        "the link must still be a link after the refused save"
    );
    assert!(
        no_scratch_files_beside(tmp.path()),
        "a refused write must not leave a scratch copy behind"
    );
}

/// The remaining shape a resolved final component can take: a link pointing at a
/// *directory*. `rename(2)` refuses to put a file over a directory, so the save
/// fails loudly and both the link and the directory are left alone — the same
/// refusal `File::create` gave (`EISDIR`). Characterization: this already held, and
/// it is asserted here so the enumeration of symlink cases the resolver answers is
/// checked rather than claimed.
#[cfg(unix)]
#[test]
fn a_symlink_to_a_directory_fails_the_save_and_keeps_both() {
    let tmp = tempfile::tempdir().unwrap();
    let directory = tmp.path().join("saves");
    fs::create_dir(&directory).unwrap();
    let link = tmp.path().join("magus.armc");
    std::os::unix::fs::symlink(&directory, &link).unwrap();

    let result = write_file_atomically(&link, "new");

    assert!(
        result.is_err(),
        "a directory is not a document to overwrite"
    );
    assert!(
        fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink(),
        "the link must survive the refused save"
    );
    assert!(directory.is_dir(), "the directory must survive it too");
    assert!(no_scratch_files_beside(tmp.path()));
}

/// Klaus K1(a): a behaviour *reversal*, not merely lost metadata. `chmod 444` on
/// a finished character used to make the save fail loudly — `File::create` on the
/// target returned `EACCES`. After the switch to rename it silently succeeded,
/// because `rename(2)` needs write permission on the *directory* and none at all
/// on the destination file. The user's own "do not overwrite this" marker became
/// a no-op, with the previous version gone and nothing said.
#[cfg(unix)]
#[test]
fn a_write_protected_target_refuses_the_write_and_keeps_its_bytes() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("finished.armc");
    fs::write(&path, "old").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o444)).unwrap();

    let result = write_file_atomically(&path, "new");

    assert!(
        result.is_err(),
        "a write-protected save must refuse the write, as it did before the \
         atomic replace — silently overwriting it makes the mode bits a lie"
    );
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "old",
        "the protected document must keep its bytes"
    );
    assert!(
        no_scratch_files_beside(tmp.path()),
        "a refused write must not leave a scratch copy behind"
    );
}

/// The other side of the same gate: an ordinary writable document must still be
/// replaced without complaint. A refusal that fired on the normal path would
/// break every save in the app.
#[cfg(unix)]
#[test]
fn an_ordinary_writable_target_is_still_replaced() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("character.armc");
    fs::write(&path, "old").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();

    write_file_atomically(&path, "new").unwrap();

    assert_eq!(fs::read_to_string(&path).unwrap(), "new");
}

/// Klaus K2: cleanup happened on exactly one path — the `Err` arm — so any death
/// of the process between creating the scratch file and the rename stranded a
/// **complete copy of the character** beside it. The window spans `sync_all`,
/// which is the slow part, so the likeliest moment to be interrupted is the
/// moment the copy is fully written. Each incident adds another, because the
/// name carries the pid and a fresh counter.
///
/// The sweep makes every successful save self-healing, which matters most on
/// Windows — a declared target, where the leading dot hides nothing and the
/// orphan sits in Explorer beside the real save with no indication of what it is.
#[test]
fn a_save_sweeps_a_scratch_file_stranded_by_an_earlier_crash() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("character.armc");
    let orphan = tmp.path().join(".character.armc.tmp-999-0");
    fs::write(&orphan, "a whole copy of the document").unwrap();

    write_file_atomically(&path, "new").unwrap();

    assert!(
        !orphan.exists(),
        "a successful save must sweep the stranded copies of its own document, \
         or they accumulate in the user's save folder forever"
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), "new");
    assert!(no_scratch_files_beside(tmp.path()));
}

/// The sweep is scoped to the document being written, and that scoping is the
/// whole safety argument for doing it at all: a scratch file belonging to some
/// *other* save is none of this write's business, and neither is an ordinary
/// file the user happens to keep beside their character.
#[test]
fn the_sweep_touches_nothing_but_the_target_s_own_scratch_files() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("character.armc");
    let other = tmp.path().join(".companion.armc.tmp-999-0");
    let user_file = tmp.path().join("character.armc.backup");
    fs::write(&other, "another document").unwrap();
    fs::write(&user_file, "the user's own file").unwrap();

    write_file_atomically(&path, "new").unwrap();

    assert_eq!(
        fs::read_to_string(&other).unwrap(),
        "another document",
        "another document's scratch file must survive"
    );
    assert_eq!(
        fs::read_to_string(&user_file).unwrap(),
        "the user's own file",
        "a file that merely shares the stem must survive"
    );
}

/// The failure path, with an orphan present: a save that cannot be performed
/// must leave the target exactly as it found it. The directory is made
/// unwritable so the scratch file cannot be created at all — the same shape as a
/// full disk or a lost network mount.
#[cfg(unix)]
#[test]
fn a_failed_write_leaves_the_target_untouched() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("character.armc");
    fs::write(&path, "old").unwrap();
    fs::set_permissions(tmp.path(), fs::Permissions::from_mode(0o555)).unwrap();

    let result = write_file_atomically(&path, "new");

    // Restore before the assertions, so a failure still lets the temp directory
    // clean itself up.
    fs::set_permissions(tmp.path(), fs::Permissions::from_mode(0o755)).unwrap();

    assert!(
        result.is_err(),
        "an unwritable directory must fail the save"
    );
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "old",
        "a failed save must leave the previous version intact — that is the \
         entire reason this module exists"
    );
}

/// Every scratch name this module makes starts with a dot and carries `.tmp-`.
fn no_scratch_files_beside(directory: &Path) -> bool {
    fs::read_dir(directory).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .contains(".tmp-")
    })
}
