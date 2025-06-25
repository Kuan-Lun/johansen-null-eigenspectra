# Build Instructions

This project requires a working C compiler toolchain because some dependencies use native code (e.g., `rayon`, `getrandom`, `libc`, `lapack`, etc.).

Follow the instructions below to install the necessary build tools for your platform.

---

## Linux / WSL (Ubuntu / Debian)

Install required packages using `apt`:

```bash
sudo apt update
sudo apt install build-essential liblapack-dev libblas-dev
```

This installs:

* A C/C++ compiler (`gcc`, `g++`)
* `make` and other build tools
* LAPACK and BLAS libraries (if used by the project)

---

## macOS

Ensure you have [Xcode Command Line Tools](https://developer.apple.com/xcode/resources/) installed:

```bash
xcode-select --install
```

This provides:

* `clang` (C compiler)
* `make`
* Standard system libraries

If you're using Homebrew for LAPACK dependencies:

```bash
brew install lapack
```

---

## Windows (Native, *Not WSL*)

This project is intended to be built in [WSL](https://learn.microsoft.com/en-us/windows/wsl/). Building natively on Windows may work with [MSVC](https://visualstudio.microsoft.com/) toolchain, but is untested.

For best results, use WSL with the instructions above.

---

## Verify

After setup, run:

```bash
cargo check
```

to confirm the build environment is ready.
