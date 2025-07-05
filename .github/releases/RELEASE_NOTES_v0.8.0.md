# Release Notes v0.8.0

## Overview

This release focuses on improving the clarity and consistency of the JohansenModel API through systematic refactoring of variant names. All computational logic remains identical, ensuring full backward compatibility at the functional level while providing clearer, more descriptive naming conventions.

## Breaking Changes

### JohansenModel Variant Renaming

The following `JohansenModel` enum variants have been renamed for improved clarity and logical consistency:

| Old Name | New Name | Model |
|----------|----------|-------|
| `InterceptNoTrendNoInterceptInCoint` | `InterceptNoTrendUnrestrictedIntercept` | Model 2 |
| `InterceptTrendWithTrendInCoint` | `InterceptTrendUnrestrictedInterceptRestrictedTrend` | Model 3 |
| `InterceptTrendNoTrendInCoint` | `InterceptTrendUnrestrictedBoth` | Model 4 |

**Migration Guide:**

- Update all references to use the new variant names
- The numeric model identifiers (0-4) remain unchanged
- All computational behavior is identical

## Improvements

### Naming Convention Enhancements

**Clearer Constraint Expression:**

- `Unrestricted` indicates terms not fully explained by cointegration relationships
- `Restricted` indicates terms fully explained by cointegration relationships  
- Removes logical contradictions in previous naming (e.g., models with trends no longer contain "NoTrend")

**Enhanced Readability:**

- Model 2: Now clearly shows it has unrestricted intercept terms
- Model 3: Explicitly indicates unrestricted intercept but restricted trend  
- Model 4: Clearly states both intercept and trend are unrestricted

## Documentation Updates

- **Library Usage Documentation**: Updated `LIBRARY_USAGE.md` with new variant names and corrected table references
- **API Documentation**: All inline documentation and comments updated to reflect new naming convention

## Testing

- **Comprehensive Test Updates**: All 13 test cases updated to use new variant names
- **Validation**: All tests pass with identical assertions and logic
- **Coverage**: No reduction in test coverage; all functionality remains fully tested

## Implementation Details

### Files Modified

**Core Implementation:**

- `src/johansen_models.rs` - Primary enum variant definitions and implementations
- `src/johansen_statistics.rs` - F matrix construction logic references  
- `src/data_storage/thread_manager.rs` - Eigenvalue counting logic references

**Testing:**

- `src/tests/johansen_models_test.rs` - Complete test suite updates

**Documentation:**

- `LIBRARY_USAGE.md` - Model variant reference table and examples

### Commit History

This release includes 4 atomic commits following conventional commit standards:

1. **refactor(johansen)**: Primary model variant naming improvements
2. **test(johansen)**: Comprehensive test suite updates  
3. **refactor(statistics)**: Statistical computation module updates
4. **docs(library)**: Library documentation updates

## Stability Guarantees

- **Functional Compatibility**: All computational results remain identical
- **Numeric Model IDs**: Model numbers (0-4) unchanged for CLI compatibility
- **Statistical Accuracy**: No changes to underlying mathematical implementations
- **Performance**: Zero performance impact from naming changes

## Technical Notes

- **Compilation**: Clean rebuild recommended to update all references
- **Dependencies**: No dependency changes
- **Binary Compatibility**: CLI interface unchanged (uses numeric model IDs)

## Migration Path

For applications using the old variant names:

```rust
// Before (v0.7.0 and earlier)
JohansenModel::InterceptNoTrendNoInterceptInCoint

// After (v0.8.0+)  
JohansenModel::InterceptNoTrendUnrestrictedIntercept

// Numeric IDs remain the same
JohansenModel::from_number(2) // Still returns the same model functionality
```

---

**Full Changelog**: [v0.7.0...v0.8.0](https://github.com/Kuan-Lun/johansen-null-eigenspectra/compare/v0.7.0...v0.8.0)
