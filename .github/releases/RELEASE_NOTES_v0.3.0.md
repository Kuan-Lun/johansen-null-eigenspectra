# Release Notes: Version 0.3.0

## Improvements

### Complete --quiet Flag Implementation

- **Fixed incomplete quiet mode**: The `--quiet` flag now properly suppresses ALL progress output
- **Enhanced output control**: Thread configuration messages now respect the quiet parameter
- **Consistent behavior**: All user-facing messages are now controlled by the --quiet flag
- **New conditional printing system**: Added robust macros for better output management

## What's Fixed

Previously, the `--quiet` flag only suppressed some output messages, leaving thread information and other progress details still visible. This release completes the implementation:

- Thread configuration messages now hidden in quiet mode
- All simulation progress output properly suppressed
- Only essential system messages remain visible
- Perfect for automation and scripting scenarios

## Usage Examples

```bash
# Before: quiet mode still showed thread info and some progress
# After: truly quiet output suitable for scripts

johansen-null-eigenspectra --dim 5 --runs 100,000 --quiet
# Now produces minimal output perfect for automation
```

## Backward Compatibility

All existing functionality remains unchanged. This is purely an enhancement to the existing `--quiet` flag behavior.

---

**Full Changelog**: [v0.2.9...v0.3.0](https://github.com/Kuan-Lun/johansen-null-eigenspectra/compare/v0.2.9...v0.3.0)
