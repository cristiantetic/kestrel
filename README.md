# Kestrel

Kestrel is an operating system in which no app can touch your data unless you hand it over, and you never see a permission pop-up. Picking a file in the file picker *is* the permission: everything the app gets comes from that act, and you can take it back at any time. It is built on three ideas: capabilities (a program can use only what it holds an unforgeable key for), WebAssembly apps (one app package runs on every CPU Kestrel supports) and the seL4 microkernel underneath.

## Status

Epic 1: the spike. It answers the project's core technical bet: can a precompiled Wasm component run under the no_std Wasmtime runtime inside a seL4 component on QEMU x86_64 and ARM64? The repository currently holds only the workspace skeleton, the dev container and the `cargo xtask` command surface; nothing boots yet.

## Quick start (dev container)

All builds run inside a pinned dev container: the Rust toolchain pinned in `rust-toolchain.toml`, the seL4 17.0.0 build dependencies, QEMU 11.1.3 and `cargo-deny`.

**VS Code:** install Docker and the Dev Containers extension, open this folder and choose *Reopen in Container*.

**Docker only:**

```sh
docker build -t kestrel-dev -f .devcontainer/Dockerfile .
docker run --rm -it --user "$(id -u):$(id -g)" -v "$PWD:/work" kestrel-dev
```

Inside the container:

```sh
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check
```

## `cargo xtask`

| Command | Purpose |
| --- | --- |
| `cargo xtask run --arch x86_64\|aarch64 [--gdb]` | Build the boot image and boot it in QEMU; `--gdb` starts QEMU's GDB stub. |
| `cargo xtask test --arch x86_64\|aarch64` | Boot the test image and report the `KTEST <name> PASS\|FAIL` results. |
| `cargo xtask bench --arch x86_64\|aarch64` | Boot the benchmark image and record the results. |

The commands are stubs until Story 1.3: each prints `<command> (<arch>): not implemented yet (Story 1.3)` and exits with code 2.

## Licence

Apache-2.0; see [LICENSE](LICENSE). `cargo deny check` rejects any Rust dependency whose licence is not on the allow-list in `deny.toml`.
