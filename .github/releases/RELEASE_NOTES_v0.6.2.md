# Release Notes: Version 0.6.2

## Overview

Version 0.6.2 is a maintenance release focused on code organization, documentation improvements, and enhanced developer experience. This release refactors the internal architecture while maintaining full backward compatibility with the public API.

## Improvements

### Code Organization & Architecture

- **Modular Analysis System**: Introduced a new `simulation_analyzers` module with functional approach replacing trait-based analyzers
- **Enhanced API Boundaries**: Improved module visibility with better encapsulation using `pub(crate)` and centralized public API exports through `lib.rs`
- **Test Infrastructure Overhaul**: Comprehensive test reorganization moving integration tests from `tests/` to `src/tests/` for better proximity to source code
- **Module Visibility**: Cleaner public API boundaries while maintaining internal accessibility for testing

### Documentation & Examples

- **Library Usage Example**: Added `examples/readme_example.rs` demonstrating library integration
- **README Updates**: Updated usage examples to match the new API structure with corrected import paths
- **Code Documentation**: Improved inline documentation and converted problematic doctests to text examples
- **API Documentation**: Marked internal implementation details with `#[doc(hidden)]` for cleaner public documentation

### Bug Fixes

- **CLI Model Consistency**: Fixed potential inconsistency in data reading demo where hardcoded model was replaced with user-selected model from CLI arguments

### Developer Experience

- **Better Test Organization**: Split large integration test files into focused, modular test suites
- **Improved Import Structure**: Fixed import paths from external crate references to proper internal `crate::` references
- **Enhanced Code Quality**: Better separation of concerns with functional approach in simulation analysis

## Migration Guide

This release maintains full backward compatibility. No changes are required for existing users of the library or CLI tool.

### For Library Users

- The core API (`EigenvalueSimulation`, `JohansenModel`) remains unchanged
- All existing functionality is preserved with improved internal organization

### For CLI Users

- All command-line options and behavior remain identical
- Users will benefit from more consistent model handling in data reading operations

## Technical Details

### Refactoring Summary

- **21 commits** since v0.6.1
- **2 pull requests** merged for major refactoring efforts
- **Files reorganized**: Test files moved to improve code proximity and enable white-box testing
- **API improvements**: Better encapsulation without breaking changes

### Conventional Commits Used

This release follows conventional commit standards with clear categorization:

- `refactor`: Internal code improvements and reorganization
- `docs`: Documentation and example improvements  
- `test`: Test infrastructure and organization changes
- `fix`: Bug fixes for consistency issues

## What's Next

This release establishes a solid foundation for future development with:

- Cleaner modular architecture ready for new features
- Improved test infrastructure supporting both unit and integration testing
- Better documentation structure for enhanced user experience
- Robust API boundaries enabling safer internal refactoring

---

**Full Changelog**: [v0.6.1...v0.6.2](https://github.com/Kuan-Lun/johansen-null-eigenspectra/compare/v0.6.1...v0.6.2)
