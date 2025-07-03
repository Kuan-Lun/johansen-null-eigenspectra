# F Matrix Construction for Johansen Models

This document explains how the F matrix is constructed for each of the five Johansen cointegration test models. The F matrix construction determines the specific form of the asymptotic null distribution being simulated.

## Background

In the Johansen cointegration test, different models correspond to different assumptions about deterministic components (intercepts and trends) and their presence in the cointegrating relationships. The F matrix captures these assumptions in the simulation.

## Model Definitions

The models follow the specifications in Johansen, S. (1996). *Likelihood-Based Inference in Cointegrated Vector Autoregressive Models*. Oxford University Press, Oxford, 2nd edition, Subsection 15.1.

### Model 0: No Intercept, No Trend

**Description**: No deterministic components in the data generating process.

**F Matrix Construction**: $F_t = B_t$

where $B_t$ is the Brownian motion matrix at time step $t$.

**Implementation**: Direct copy of the Brownian motion matrix.

### Model 1: Intercept, No Trend, Intercept in Cointegration

**Description**: Constant term is present and is included in the cointegrating relationships.

**F Matrix Construction**: $F_{t}' = (B_{t}',1)$

where the bottom row is filled with ones (constant term).

**Implementation**: Augment the Brownian motion matrix with a row of ones.

### Model 2: Intercept, No Trend, Intercept Not Fully Explained by Cointegration

**Description**: Constant term is present but not fully captured by the cointegrating relationships.

**F Matrix Construction**:

- Demean the first (dim-1) rows of the Brownian motion matrix
- Construct time trend: $y_t = t/T - 0.5$
- Combine demeaned data with the time trend

**Implementation**: More complex transformation involving demeaning and trend construction.

### Model 3: Intercept, Trend, Trend in Cointegration

**Description**: Both constant and linear trend are present, with the trend included in the cointegrating relationships.

**F Matrix Construction**:

- Demean all rows of the Brownian motion matrix
- Construct time trend: $y_t = t/T - 0.5$
- Augment the demeaned matrix with the time trend

**Implementation**: Demeaning followed by trend augmentation.

### Model 4: Intercept, Trend, Not Fully Explained by Cointegration

**Description**: Both constant and linear trend are present, but neither is fully captured by the cointegrating relationships.

**F Matrix Construction**:

- Construct time trend: $y_t = t/T$
- Create quadratic trend: $y_t^2$
- Project out constant and linear trend from the augmented data matrix
- Return the residual matrix

**Implementation**: Most complex construction involving projection and residual computation.

## Code Implementation

The actual implementation of these constructions can be found in the `construct_f_matrix` function in `src/johansen_statistics.rs`. Each model case is handled separately with detailed comments explaining the mathematical operations.

## Usage in Simulation

During the eigenvalue simulation:

1. For each time step $t$, the Brownian motion $B_t$ is used to construct $F_t$
2. The F matrix construction depends on the selected model (0-4)
3. The constructed F matrices are then used in the generalized eigenvalue problem to compute the test statistics

The choice of model affects both the dimensionality and the structure of the matrices involved in the eigenvalue computation, leading to different asymptotic null distributions for each model.
