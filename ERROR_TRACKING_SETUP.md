# Automatic Error Tracking Setup

## Overview

This project now includes a procedural macro system for automatically tracking function names in error backtraces. Instead of manually appending function names to error contexts, the `#[track_errors]` macro does this automatically.

## What's Been Set Up

### 1. Cargo.toml Changes
- Added `proc-macro = true` to enable procedural macro support
- Added dependencies: `syn`, `quote`, and `proc-macro2`

### 2. New Error System Structure
- **Backtrace Format**: Changed from `String` to `Vec<String>` to store a list of function names
- **WithBacktrace Trait**: Updated to include `add_context(&mut self, function_name: String)` method
- **All Error Types**: Updated `AuthError`, `ConfigError`, `GitError`, and `NormalisedPathError` to use the new system

### 3. Procedural Macros (`src/lib.rs`)
Two macros are available:

#### `#[track_errors]`
Automatically adds the function name to error backtraces when errors are created or propagated.

#### `#[track_errors_with_name("custom_name")]`
Like `#[track_errors]` but uses a custom name instead of the function name.

### 4. Updated Error Display
The `print_error_chain` function in `utils.rs` now displays the call stack as a numbered list.

## How to Use

### Basic Usage
```rust
use crate::track_errors;

#[track_errors]
fn my_function() -> Result<String, Box<dyn CustomErrorTrait>> {
    // Normal function code with ? operator
    let result = some_operation()?;
    another_operation()?;
    Ok(result)
}
```

### Custom Function Names
```rust
#[track_errors_with_name("database_operation")]
fn complex_database_function_name() -> Result<Data, Box<dyn CustomErrorTrait>> {
    // Errors will show "database_operation" instead of "complex_database_function_name"
    Ok(data)
}
```

### Error Output Example
When an error occurs through multiple function calls, you'll see:
```
Error: Something went wrong at the lowest level
Call stack:
  1: inner_function
  2: middle_function
  3: outer_function
```

## Current Status

✅ **Completed**:
- Cargo.toml configuration
- Procedural macro implementation
- Error system updates (Vec<String> backtrace)
- All existing error types updated
- Error display formatting

⚠️ **Known Issues**:
- The procedural macro has some type inference issues in complex scenarios
- Examples need refinement for edge cases

## Integration into Existing Code

To start using this system in your existing functions:

1. **Import the macro**: `use crate::track_errors;` (or `use bitpet_cli::track_errors;` from external crates)
2. **Add the attribute**: `#[track_errors]` above function definitions
3. **Ensure return type**: Function must return `Result<T, Box<dyn CustomErrorTrait>>`
4. **Use ? operator**: Normal error propagation with `?` works automatically

## Benefits

- **Automatic**: No manual context addition needed
- **Zero runtime cost**: When no errors occur, there's no overhead
- **Comprehensive**: Shows complete call stack when errors occur
- **Flexible**: Can be applied selectively to functions
- **Compatible**: Works with existing error handling patterns

## Technical Details

### Macro Implementation
The macro works by:
1. Capturing the function name at compile time
2. Wrapping the function body in error-handling logic
3. Automatically calling `add_context()` on any errors that are returned
4. Supporting both sync and async functions

### Error Trait Requirements
For the macro to work, error types must implement:
- `std::error::Error`
- `WithBacktrace` trait with `backtrace()` and `add_context()` methods
- `CustomErrorTrait` (which combines the above)

This setup provides a robust foundation for automatic error tracking throughout the codebase.
