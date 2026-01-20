# Broken Tests Archive

These test files were moved here because they contain references to modules that no longer exist in the codebase, making them impossible to compile.

## Files

### buff_display_test.rs
- **Issue**: References modules that don't exist
- **Status**: Needs to be rewritten to use current API
- **Purpose**: Tests buff display functionality

### simulation_test.rs  
- **Issue**: Uses deprecated simulation functions and missing modules
- **Status**: Needs significant refactoring
- **Purpose**: Tests core simulation logic

### storage_test.rs
- **Issue**: References `storage` and `storage_manager` modules that were removed
- **Status**: Obsolete - storage system was replaced
- **Purpose**: Tests storage/persistence functionality

## Recommended Actions

1. **buff_display_test.rs**: Rewrite using current buff system and display APIs
2. **simulation_test.rs**: Extract useful test cases and integrate into current test suite
3. **storage_test.rs**: Discard - functionality replaced by different system

## Issue Tracking

See issue #7 for tracking the resolution of these broken tests.</content>
<parameter name="filePath">simulation-wasm/tests/broken_tests/README.md