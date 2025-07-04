# Johansen Null Eigenspectra

This project performs large scale Monte Carlo simulations of the eigenvalues that appear in Johansen's null distribution. Results are written to `*.dat` files which can be reloaded later for analysis.

## Quick Start

### Option 1: Download Binary (Recommended)

1. Go to the [Releases page](https://github.com/Kuan-Lun/johansen-null-eigenspectra/releases)
2. Download the binary for your platform
3. Run the help command: `./johansen-null-eigenspectra --help`

### Option 2: Build from Source

Requires a working C compiler and LAPACK. See [BUILD.md](./BUILD.md) for compiler setup and [LAPACK_SETUP.md](./LAPACK_SETUP.md) if your system does not already provide LAPACK/BLAS.

```bash
# For source builds (all platforms)
cargo build --release

# Run the built binary:
./target/release/johansen-null-eigenspectra --help
```

## Command line interface

The binary accepts several options. The following list mirrors the help output from `src/cli.rs`:

```text
--threads <int>      number of threads for parallel computation (default: number of logical cores)
--steps <int>        number of simulation steps (default: 10,000)
--runs <int>         number of runs per model (default: 10,000,000)
--dim-start <int>    starting matrix dimension (default: 1)
--dim-end <int>      ending matrix dimension (default: 12)
--dim <int>          run a single dimension (sets start and end to the same value)
--model <list>       comma separated list of model numbers to compute (default: 0,1,2,3,4)
--quiet              suppress progress output
-h, --help           show this help message
-v, --version        show version information
```

## Theoretical Background

The eigenvalues computed in this simulation correspond to the asymptotic null distribution of the Johansen cointegration test.

### Theory

The theoretical eigenvalues are the values of ρ that solve:

```math
\det {\left( \rho \int_{0}^{1} F F' d u - {\left( \int_{0}^{1} F (dB)' \right)} {\left( \int_{0}^{1} (d B) F' \right)} \right)} = 0
```

where:

- $\det$ is the determinant operator
- $B$ is a standard Brownian motion process (dimension corresponds to the `--dim` CLI parameter)
- $F$ is constructed according to the specific Johansen model, following the definitions in Johansen, S. (1996). *Likelihood-Based Inference in Cointegrated Vector Autoregressive Models*. Oxford University Press, Oxford, 2nd edition, Subsection 15.1.

### Implementation

The estimated eigenvalues $\hat{\rho}$ are computed by solving the discrete approximation:

```math
\det {\left( \hat{\rho} \sum_{t=1}^{T} F_{t-1} F_{t-1}' - {\left( \sum_{t=1}^{T} F_{t-1} {\left( B_{t}-B_{t-1} \right)} ' \right)} {\left( \sum_{t=1}^{T} {\left( B_{t}-B_{t-1} \right)} F_{t-1}' \right)} \right)} = 0
```

where:

- $T$ is the total number of simulation steps (corresponds to the `--steps` CLI parameter)
- $B_{t}$ represents the discretized Brownian motion at time step $t$ (dimension = `--dim` parameter)
- $F_{t}$ is the F matrix constructed from the Brownian motion at time step $t$
- The construction of $F$ depends on the specific Johansen model being tested

For detailed information about how the F matrix is constructed for each model, see [F_MATRIX.md](./F_MATRIX.md).

## Usage Examples

This example runs the simulation for dimension 5 with 5,000 steps and 1,000,000 runs per model using 4 threads:

```bash
./johansen-null-eigenspectra --threads 4 --steps 5,000 --runs 1,000,000 --dim 5

# Building from source
cargo run --release -- --threads 4 --steps 5,000 --runs 1,000,000 --dim 5
```

**Note**: Numeric arguments support comma separators for better readability (e.g., `--runs 1,000,000` or `--runs 1000000`).

### Model Numbers

The `--model` parameter accepts comma-separated model numbers (0-4). Each number corresponds to a specific Johansen cointegration test model:

| Model | Description |
|-------|-------------|
| 0 | No intercept, no trend |
| 1 | Intercept, no trend, intercept in cointegration |
| 2 | Intercept, no trend, intercept not fully explained by cointegration |
| 3 | Intercept, trend, trend in cointegration |
| 4 | Intercept, trend, intercept and trend not fully explained by cointegration |

**Examples:**

- `--model 0,2` runs only models 0 and 2
- `--model 1` runs only model 1
- If not specified, all models (0,1,2,3,4) are computed by default

The simulation writes results to `data/eigenvalues_modelX_dimY_stepsZ.dat` where `X` is the model number, `Y` is the dimension, and `Z` is the number of steps.

## Data File Format

The simulation results are stored in a custom binary format optimized for high-performance writing and reading of large-scale simulation data. For detailed information about the file structure, including byte-level format specifications, see [DATA_FORMAT.md](./DATA_FORMAT.md).

**Key features of the data format:**

- Efficient append-only writing with resume capability
- Built-in data integrity verification
- Optimized for large files (millions of records)
- Cross-platform compatibility (little-endian encoding)

For details on using this crate as a library, including example code, see [LIBRARY_USAGE.md](./LIBRARY_USAGE.md).
