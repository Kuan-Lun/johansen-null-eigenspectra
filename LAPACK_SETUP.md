# System LAPACK Integration Guide

This project, **Johansen Null Eigenspectra**, relies on the system's LAPACK implementation for generalized eigenvalue computations (e.g., `dggev_`). It links to platform-specific LAPACK backends via `.cargo/config.toml`.

This document explains how to install and configure LAPACK on different platforms to ensure the project builds correctly.

---

## Prerequisites

* [Rust](https://rustup.rs) (installed via `rustup`)
* A C compiler (e.g., GCC, Clang, or MSVC)
* Git (for fetching dependencies)
* LAPACK (system-provided or manually installed)

---

## Platform Setup

### Linux

Install LAPACK from your system package manager:

```bash
sudo apt update
sudo apt install liblapack-dev
```

This installs `liblapack.so` (typically located at `/usr/lib/x86_64-linux-gnu/`) with the required symbol `dggev_`.

No additional configuration is needed—`cargo build` should work out of the box.

---

### macOS

macOS provides LAPACK through Apple’s **Accelerate** framework.

Ensure you have the command-line tools installed:

```bash
xcode-select --install
```

Then, configure `.cargo/config.toml` with:

```toml
[target.x86_64-apple-darwin]
rustflags = ["-framework", "Accelerate"]
```

No additional dependencies are needed.

---

## Building the Project

Once your LAPACK setup is complete:

```bash
cargo build --release
```

---

## Troubleshooting

If you see linker errors such as:

```bash
undefined reference to `dggev_`
```

Verify:

* The correct `.so` or `.dylib` is present and in the expected location
* The linker flags in `.cargo/config.toml` are correct for your platform
* You are using the correct Rust toolchain

---

## Notes

* This project intentionally does **not** use `lapack-src` to ensure it links against optimized system LAPACK libraries.
* Platform-specific linker flags are handled via `.cargo/config.toml` instead of modifying `build.rs`.
