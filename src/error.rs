pub trait WithBacktrace {
    fn backtrace(&self) -> &Vec<String>;
    fn add_context(&mut self, function_name: String);
}

pub trait CustomErrorTrait: std::error::Error + WithBacktrace {}

/// Generic wrapper for any error type that adds backtrace support
#[derive(Debug)]
pub struct ErrorWithBacktrace<T> {
    error: T,
    backtrace: Vec<String>,
}

impl<T> ErrorWithBacktrace<T> {
    pub fn new(error: T) -> Self {
        Self {
            error,
            backtrace: Vec::new(),
        }
    }
}

impl<T: std::fmt::Display> std::fmt::Display for ErrorWithBacktrace<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl<T: std::error::Error + 'static> std::error::Error for ErrorWithBacktrace<T> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

impl<T> WithBacktrace for ErrorWithBacktrace<T> {
    fn backtrace(&self) -> &Vec<String> {
        &self.backtrace
    }

    fn add_context(&mut self, function_name: String) {
        self.backtrace.push(function_name);
    }
}

impl<T: std::error::Error + 'static> CustomErrorTrait for ErrorWithBacktrace<T> {}

/// Macro to generate From implementations for error types
macro_rules! impl_custom_error_from {
    ($error_type:ty) => {
        impl From<$error_type> for Box<dyn CustomErrorTrait> {
            fn from(error: $error_type) -> Self {
                Box::new(ErrorWithBacktrace::new(error))
            }
        }
    };
}

/// Wrapper for String to make it implement Error
#[derive(Debug)]
pub struct StringError(pub String);

impl std::fmt::Display for StringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for StringError {}

// Special implementation for String since it doesn't implement Error
impl From<String> for Box<dyn CustomErrorTrait> {
    fn from(error: String) -> Self {
        Box::new(ErrorWithBacktrace::new(StringError(error)))
    }
}

// Generate From implementations for error types that already implement Error
impl_custom_error_from!(reqwest_middleware::Error);
impl_custom_error_from!(reqwest::Error);
impl_custom_error_from!(serde_json::Error);
