//! `cargo xtask`: the one command that builds, boots, tests and benchmarks Kestrel.
//!
//! Story 1.1 provides only the command-line surface. Story 1.3 implements the commands.

use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};

/// Exit code of a command that is not implemented yet.
const EXIT_NOT_IMPLEMENTED: u8 = 2;

#[derive(Parser)]
#[command(
    name = "cargo xtask",
    about = "Build, boot, test and benchmark Kestrel"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Build the boot image and boot it in QEMU.
    Run {
        /// Target architecture.
        #[arg(long, value_enum)]
        arch: Arch,
        /// Start QEMU's GDB stub.
        #[arg(long)]
        gdb: bool,
    },
    /// Boot the test image and report the `KTEST` results.
    Test {
        /// Target architecture.
        #[arg(long, value_enum)]
        arch: Arch,
    },
    /// Boot the benchmark image and record the results.
    Bench {
        /// Target architecture.
        #[arg(long, value_enum)]
        arch: Arch,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum Arch {
    #[value(name = "x86_64")]
    X86_64,
    #[value(name = "aarch64")]
    Aarch64,
}

impl Arch {
    fn name(self) -> &'static str {
        match self {
            Arch::X86_64 => "x86_64",
            Arch::Aarch64 => "aarch64",
        }
    }
}

fn not_implemented(command: &str, arch: Arch) -> ExitCode {
    eprintln!(
        "{command} ({}): not implemented yet (Story 1.3)",
        arch.name()
    );
    ExitCode::from(EXIT_NOT_IMPLEMENTED)
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::Run { arch, gdb: _ } => not_implemented("run", arch),
        Command::Test { arch } => not_implemented("test", arch),
        Command::Bench { arch } => not_implemented("bench", arch),
    }
}
