# Release Notes: Version 0.5.0

## Major Features

### Enhanced Data Storage with Format V5

- **Upgraded file format** from V4 to V5 with comprehensive parameter validation
- **Added header metadata** including model, dimension, and steps parameters for data integrity
- **Intelligent file validation** that automatically recreates incompatible files from previous versions
- **Improved error handling** with contextual information showing model/dim/steps mismatches

### CLI Improvements

- **New version flag**: Added `--version` and `-v` options to display application version
- **Better user experience** for checking software version

## Bug Fixes

- **Fixed remaining time estimation** in resume mode to use current session progress instead of total progress
- **More accurate progress tracking** when resuming interrupted simulations

## Breaking Changes

- **File format upgrade**: Existing V4 data files will be automatically recreated when accessed
- **API changes**: `AppendOnlyWriter::new()` removed in favor of unified `with_expected_size()` API
- **Return type change**: `read_append_file()` now returns `(data, model, dim, steps)` tuple instead of just data

## Migration Guide

- **Automatic migration**: Old V4 files will be automatically recreated when accessed
- **No manual action required**: The upgrade process is transparent to users
- **Data preservation**: All existing data will be preserved during the upgrade process

## Testing & Quality

- **Comprehensive test updates** to validate new header parameters
- **Enhanced error message clarity** with contextual model/dimension/steps information
- **Improved validation logic** to prevent file misuse across different simulation parameters

---

**Full Changelog**: [v0.4.0...v0.5.0](https://github.com/Kuan-Lun/johansen-null-eigenspectra/compare/v0.4.0...v0.5.0)
