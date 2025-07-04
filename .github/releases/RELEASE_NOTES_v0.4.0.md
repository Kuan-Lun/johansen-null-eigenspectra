# Release Notes: Version 0.4.0

## Key Highlights

- **~10% File Size Reduction**: Combined storage optimizations reduce simulation file sizes significantly
- **Enhanced Performance**: Aggressive compiler optimizations for maximum runtime performance
- **Improved UX**: Better progress reporting with formatted numbers for large-scale simulations

## Storage Optimizations

- **Seed Storage Optimization**: Reduced seed field from `u64` to `u32` (50% size reduction per field)
- **Eigenvalue Count Optimization**: Compressed from `u32` to `u8` (75% size reduction per field)  
- **Combined Impact**: ~7.3% overall storage reduction for typical simulation files

## Performance Improvements

- **Aggressive Compiler Optimizations**:
  - Link-time optimization (LTO) enabled
  - Maximum optimization level (`opt-level = 3`)
  - Reduced codegen units for better optimization
  - Overflow checks disabled for release builds
  - Panic strategy set to `abort` for smaller binaries

## Type Safety & Reliability

- **Enhanced Validation**: Tightened eigenvalue count limits from 1000 to 255 (more realistic bounds)
- **Overflow Protection**: Added safeguards for eigenvalue_count exceeding u8::MAX
- **Improved Error Handling**: Better data integrity checks with panic on magic header mismatch
- **Zero-Cost Conversions**: Safe u32→u64 conversion maintains RNG compatibility

## User Experience

- **Formatted Progress Display**: Added thousand separators to progress reporting (e.g., "1,000,000" instead of "1000000")
- **Better Readability**: Enhanced checkpoint resume information for large-scale simulations
- **Consistent API**: Updated all function signatures for improved type consistency

## Development Infrastructure

- **DevContainer Improvements**: Added git auto-sync for consistent development environment
- **Comprehensive Testing**: Added boundary condition tests and overflow protection validation

## Technical Details

- **File Format**: EIGENVALS_V2 → V3 → V4 (automatic migration)
- **API Changes**: Seed parameters changed from `u64` to `u32` across all functions
- **Memory Footprint**: Reduced per-record storage overhead

## Migration Notes

- **Automatic**: Existing data files will be automatically migrated on first access
- **No Action Required**: All format upgrades happen transparently
- **Performance**: Zero-cost abstractions maintain computational efficiency

---

**Full Changelog**: [v0.3.0...v0.4.0](https://github.com/Kuan-Lun/johansen-null-eigenspectra/compare/v0.3.0...v0.4.0)
