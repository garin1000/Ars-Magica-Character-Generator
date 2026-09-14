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

/// A dangling link has no file to write through, so there is nothing to follow
/// and the write lands on the link's own path. The point of the test is that it
/// does not *fail*: `fs::canonicalize` errors on a link that points nowhere, and
/// a save that surfaced that error would be refusing to write a document it is
/// perfectly able to write.
#[cfg(unix)]
#[test]
fn a_dangling_symlink_does_not_stop_the_save() {
    let tmp = tempfile::tempdir().unwrap();
    let link = tmp.path().join("magus.armc");
    std::os::unix::fs::symlink(tmp.path().join("gone.armc"), &link).unwrap();

    write_file_atomically(&link, "new").unwrap();

    assert_eq!(fs::read_to_string(&link).unwrap(), "new");
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
