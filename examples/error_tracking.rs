//! Example demonstrating the automatic error tracking system
//!
//! This example shows how to use the #[track_errors] procedural macro
//! to automatically track function names in error backtraces.
//!
//! Since this is a proc-macro crate, we need to create a more complex
//! setup to demonstrate the functionality.

use bitpet_cli::track_errors;

// For this example, we'll define a simple error type that implements the required traits
#[derive(Debug)]
struct SimpleError {
    message: String,
    backtrace: Vec<String>,
}

impl std::fmt::Display for SimpleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SimpleError {}

impl SimpleError {
    fn new(message: String) -> Self {
        Self {
            message,
            backtrace: Vec::new(),
        }
    }
}

// Implement the required traits for our error type
trait WithBacktrace {
    fn backtrace(&self) -> &Vec<String>;
    fn add_context(&mut self, function_name: String);
}

impl WithBacktrace for SimpleError {
    fn backtrace(&self) -> &Vec<String> {
        &self.backtrace
    }

    fn add_context(&mut self, function_name: String) {
        self.backtrace.push(function_name);
    }
}

trait CustomErrorTrait: std::error::Error + WithBacktrace {}
impl CustomErrorTrait for SimpleError {}

impl From<String> for Box<dyn CustomErrorTrait> {
    fn from(msg: String) -> Self {
        Box::new(SimpleError::new(msg))
    }
}

// Now demonstrate the macro usage
#[track_errors]
fn level_three() -> Result<String, Box<dyn CustomErrorTrait>> {
    // This will create an error at the deepest level
    let error: Box<dyn CustomErrorTrait> = "Something went wrong at level 3".to_string().into();
    Err(error)
}

#[track_errors]
fn level_two() -> Result<String, Box<dyn CustomErrorTrait>> {
    // This will propagate the error from level_three and add its own context
    let result = level_three()?;
    Ok(result)
}

#[track_errors]
fn level_one() -> Result<String, Box<dyn CustomErrorTrait>> {
    // This will propagate the error from level_two and add its own context
    let result = level_two()?;
    Ok(result)
}

fn main() {
    println!("🔍 Testing automatic error tracking...\n");

    match level_one() {
        Ok(result) => println!("Success: {}", result),
        Err(error) => {
            println!("Error: {}", error);

            let backtrace = error.backtrace();
            if !backtrace.is_empty() {
                println!("Call stack:");
                for (i, func_name) in backtrace.iter().enumerate() {
                    println!("  {}: {}", i + 1, func_name);
                }
            }
        }
    }

    println!("\n✨ Notice how the call stack shows:");
    println!("   1. level_three (where the error was created)");
    println!("   2. level_two (propagated the error)");
    println!("   3. level_one (propagated the error)");
    println!("\nThis happens automatically with the #[track_errors] macro!");
}
