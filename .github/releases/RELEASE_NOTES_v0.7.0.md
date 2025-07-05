# Release Notes: Version 0.7.0

## Overview

Version 0.7.0 introduces significant improvements in project organization, documentation structure, and code quality. This release focuses on enhanced developer experience through modular documentation, test restructuring, and API refinements while maintaining full backward compatibility.

## Major Improvements

### Documentation & Project Structure

- **Modular AI Agent Guidelines**: Complete restructuring of project guidelines into specialized files:
  - Created `.agents/` directory with task-specific guidance files
  - `code-generation.md` for programming standards and practices
  - `testing.md` for test strategies and statistical validation  
  - `git-workflow.md` for version control and commit conventions
  - `documentation.md` for documentation writing standards
  - `performance.md` for optimization guidelines and benchmarking
- **Enhanced Library Documentation**: Added comprehensive `LIBRARY_USAGE.md` with detailed API usage examples
- **Simplified README**: Streamlined main README to focus on essential information while directing users to specialized documentation
- **Example Improvements**: Updated `library_usage_example.rs` with more practical parameters for faster testing

### Code Organization & Architecture

- **Centralized Test Structure**: Moved all `data_storage` tests from embedded modules to centralized `src/tests/` directory
  - Relocated append writer, uleb128, and integration tests
  - Improved test discoverability and maintenance
  - Better separation of test code from production modules
- **Data Storage API Enhancement**:
  - Simplified filename convention removing `num_runs` parameter: `eigenvalues_model{}_dim{}_steps{}.dat`
  - Added `read_all_data()` method for unrestricted data access
  - Enhanced `read_data()` with strict validation and actionable error messages

### Code Quality & Maintenance

- **Style Improvements**: Applied Clippy suggestions for modern Rust formatting
  - Converted to inlined format arguments in `display_utils.rs`
  - Fixed `clippy::uninlined_format_args` warnings
- **Cleanup**: Removed redundant temporary files and consolidated example code

## API Changes

### Data Storage Module

- **Filename Format Change**: Simplified from `eigenvalues_model{}_dim{}_steps{}_{}.dat` to `eigenvalues_model{}_dim{}_steps{}.dat`
- **New Method**: `read_all_data()` - Retrieves all available records without validation
- **Enhanced Method**: `read_data()` - Now includes comprehensive validation with helpful error messages

## Backward Compatibility

This release maintains full backward compatibility for the core statistical computation APIs. The data storage filename format change affects only new data files; existing files continue to work normally.

## Migration Guide

### For Library Users

- No code changes required for existing statistical computation functionality
- New data files will use the simplified filename format automatically
- Existing data files continue to work without modification

### For Contributors

- Review the new modular documentation structure in `.agents/` directory
- Follow updated guidelines for specific development tasks
- Tests are now centralized in `src/tests/` directory

## Technical Details

### Dependencies

No changes to external dependencies. All existing dependencies remain unchanged:

- `nalgebra` and `nalgebra-lapack` for matrix operations
- `rand` ecosystem for random number generation  
- `rayon` for parallel computation

### Performance

No performance regressions. All optimizations from previous releases remain active:

- Link-time optimization (LTO) enabled
- Maximum optimization level (`opt-level = 3`)
- Single codegen unit for better optimization
- Panic abort strategy for reduced binary size

## Testing

All existing tests continue to pass. The test restructuring improves maintainability without affecting functionality.

## Contributors

Special thanks to all contributors who helped improve the project structure and documentation quality in this release.
