# Release Notes: Version 0.2.6

This is the **initial public release** of Johansen Null Eigenspectra.

## Features

- High-performance Monte Carlo simulation for Johansen cointegration test eigenvalues
- Support for all 5 Johansen model types (0-4)
- Multi-threaded parallel computation using Rust and LAPACK
- Configurable simulation parameters (dimensions, steps, runs)
- Cross-platform binaries for Linux and macOS
- Data export to `.dat` files for further analysis
- Command-line interface with comprehensive options

## Installation

Download the appropriate binary for your platform from the assets below.

## Usage & Documentation

See the [README](https://github.com/Kuan-Lun/johansen-null-eigenspectra/blob/main/README.md) for detailed usage instructions, examples, and documentation.

## Requirements

**For pre-compiled binaries:**

- **Linux**: Requires system LAPACK/BLAS libraries (usually pre-installed or available via package manager)
- **macOS**: No additional dependencies (uses built-in Accelerate framework)

**For building from source:**

- Rust toolchain, C compiler, and LAPACK development libraries
