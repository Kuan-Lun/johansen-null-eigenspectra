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

### Windows (MSVC)

On Windows, the recommended approach is to install OpenBLAS via [vcpkg](https://vcpkg.io).

#### Step 1: Install vcpkg

```powershell
git clone https://github.com/microsoft/vcpkg.git
cd vcpkg
.\bootstrap-vcpkg.bat
```

> Requires **Visual Studio Build Tools** with the "Desktop development with C++" workload.

#### Step 2: Install OpenBLAS

```powershell
.\vcpkg install openblas:x64-windows
```

This installs `openblas.lib` into:

```bash
<vcpkg-root>\installed\x64-windows\lib\openblas.lib
```

#### Step 3: Project Configuration

1. Copy `openblas.lib` to the same directory where you will run `cargo build`.
2. Add the following to your `.cargo/config.toml`:

```toml
[target.x86_64-pc-windows-msvc]
rustflags = ["-L.", "-lopenblas"]
```

This tells Rust to link `openblas.lib` from the current directory.

#### Step 4: Confirm MSVC Toolchain

```sh
rustup show
```

Ensure the default host is `x86_64-pc-windows-msvc`.
If not, switch using:

```sh
rustup default stable-x86_64-pc-windows-msvc
```

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

* The correct `.so`, `.dylib`, or `.lib` is present and in the expected location
* The linker flags in `.cargo/config.toml` are correct for your platform
* You are using the correct Rust toolchain (e.g., MSVC on Windows)

---

## Notes

* This project intentionally does **not** use `lapack-src` to ensure it links against optimized system LAPACK libraries.
* Platform-specific linker flags are handled via `.cargo/config.toml` instead of modifying `build.rs`.
