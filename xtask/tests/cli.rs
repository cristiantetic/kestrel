//! Integration tests for the rows of Story 1.1's I/O matrix.
//!
//! The CLI rows run the built `xtask` binary. The GPL row runs `cargo deny` against
//! `tests/fixtures/gpl`, a separate workspace whose dependency graph holds a crate
//! declared `GPL-3.0-only`, using the repository's own `deny.toml`.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const ARCHES: [&str; 2] = ["x86_64", "aarch64"];
const COMMANDS: [&str; 3] = ["run", "test", "bench"];

fn xtask(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(args)
        .output()
        .expect("failed to start the xtask binary")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// Matrix row "Valid stub": prints `<command> (<arch>): not implemented yet (Story 1.3)`
/// on stderr and exits with code 2.
fn assert_stub(args: &[&str], command: &str, arch: &str) {
    let output = xtask(args);
    assert_eq!(
        stderr(&output),
        format!("{command} ({arch}): not implemented yet (Story 1.3)\n"),
        "stderr of `xtask {}`",
        args.join(" ")
    );
    assert!(
        output.stdout.is_empty(),
        "stdout of `xtask {}`",
        args.join(" ")
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "exit code of `xtask {}`",
        args.join(" ")
    );
}

#[test]
fn valid_stub_reports_not_implemented_for_every_command_and_arch() {
    for command in COMMANDS {
        for arch in ARCHES {
            assert_stub(&[command, "--arch", arch], command, arch);
        }
    }
}

#[test]
fn valid_stub_run_with_gdb_reports_not_implemented() {
    for arch in ARCHES {
        assert_stub(&["run", "--gdb", "--arch", arch], "run", arch);
        assert_stub(&["run", "--arch", arch, "--gdb"], "run", arch);
    }
}

#[test]
fn bad_arch_is_a_usage_error_listing_supported_arches() {
    for command in COMMANDS {
        let output = xtask(&[command, "--arch", "riscv"]);
        let err = stderr(&output);
        assert!(
            err.contains("invalid value 'riscv' for '--arch <ARCH>'"),
            "stderr of `xtask {command} --arch riscv`: {err}"
        );
        assert!(
            err.contains("[possible values: x86_64, aarch64]"),
            "stderr of `xtask {command} --arch riscv`: {err}"
        );
        assert!(
            !err.contains("not implemented"),
            "the stub must not run: {err}"
        );
        assert_eq!(
            output.status.code(),
            Some(2),
            "exit code of `xtask {command} --arch riscv`"
        );
    }
}

#[test]
fn missing_arch_is_a_usage_error() {
    for command in COMMANDS {
        let output = xtask(&[command]);
        let err = stderr(&output);
        assert!(
            err.contains("the following required arguments were not provided"),
            "stderr of `xtask {command}`: {err}"
        );
        assert!(
            err.contains("--arch <ARCH>"),
            "stderr of `xtask {command}`: {err}"
        );
        assert!(
            !err.contains("not implemented"),
            "the stub must not run: {err}"
        );
        assert_eq!(
            output.status.code(),
            Some(2),
            "exit code of `xtask {command}`"
        );
    }
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives one level below the repository root")
        .to_path_buf()
}

fn cargo() -> Command {
    Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
}

/// Matrix row "Valid stub", reached through the documented `cargo xtask` alias in
/// `.cargo/config.toml` rather than the built binary.
#[test]
fn cargo_xtask_alias_runs_the_stub() {
    // A separate target dir: the outer `cargo test` holds the lock on its own.
    let target_dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("alias-target");
    let output = cargo()
        .current_dir(repo_root())
        .env("CARGO_TARGET_DIR", &target_dir)
        .args(["--quiet", "--offline", "xtask", "run", "--arch", "x86_64"])
        .output()
        .expect("failed to start cargo");
    assert_eq!(
        stderr(&output),
        "run (x86_64): not implemented yet (Story 1.3)\n",
        "stderr of `cargo xtask run --arch x86_64`"
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "exit code of `cargo xtask run --arch x86_64`"
    );
}

/// Panics with a clear message when the `cargo deny` subcommand is not available.
fn require_cargo_deny() {
    let available = cargo()
        .args(["deny", "--version"])
        .output()
        .is_ok_and(|output| output.status.success());
    assert!(
        available,
        "cargo-deny is not installed, so the licence checks cannot run. \
         Run the tests inside the kestrel-dev container, which has the pinned cargo-deny."
    );
}

/// The real workspace passes cargo-deny. Advisories are left out because they need
/// the advisory database. Not `--offline`: cargo-deny's `cargo metadata` covers every
/// platform, so it may download locked sources (e.g. Windows-only crates) that a Linux
/// build never fetched.
#[test]
fn workspace_passes_cargo_deny_licenses_bans_sources() {
    require_cargo_deny();
    let root = repo_root();
    let output = cargo()
        .arg("deny")
        .arg("--manifest-path")
        .arg(root.join("Cargo.toml"))
        .arg("--config")
        .arg(root.join("deny.toml"))
        .args([
            "--color", "never", "--locked", "check", "licenses", "bans", "sources",
        ])
        .output()
        .expect("failed to start cargo deny");
    assert!(
        output.status.success(),
        "cargo deny rejected the workspace:\n{}",
        stderr(&output)
    );
}

#[test]
fn gpl_dependency_fails_cargo_deny_licenses() {
    require_cargo_deny();
    let root = repo_root();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/gpl/Cargo.toml");

    // --locked: the fixture's committed Cargo.lock is used and never rewritten.
    // --offline: the fixture has no registry dependencies, so no network is needed.
    let output = cargo()
        .arg("deny")
        .arg("--manifest-path")
        .arg(&fixture)
        .arg("--config")
        .arg(root.join("deny.toml"))
        .args([
            "--color",
            "never",
            "--locked",
            "--offline",
            "check",
            "licenses",
        ])
        .output()
        .expect("failed to start cargo deny");
    let err = stderr(&output);

    assert!(
        !output.status.success(),
        "cargo deny accepted a GPL dependency:\n{err}"
    );
    assert!(
        err.contains("rejected"),
        "cargo deny failed, but not with a licence rejection:\n{err}"
    );
    assert!(
        err.contains("gpl-fixture v0.0.0") && err.contains("GPL-3.0-only"),
        "cargo deny output does not name the GPL crate:\n{err}"
    );
}
