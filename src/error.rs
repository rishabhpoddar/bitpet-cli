pub trait WithBacktrace {
    fn backtrace(&self) -> String;
}

pub trait CustomErrorTrait: std::error::Error + WithBacktrace {}

impl From<String> for Box<dyn CustomErrorTrait> {
    fn from(error: String) -> Self {
        Box::new(StringError(
            error,
            std::backtrace::Backtrace::capture().to_string(),
        ))
    }
}

#[derive(Debug)]
struct StringError(String, String);

impl std::fmt::Display for StringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for StringError {}

impl WithBacktrace for StringError {
    fn backtrace(&self) -> String {
        self.1.clone()
    }
}

impl CustomErrorTrait for StringError {}

#[derive(Debug)]
struct ReqwestMiddlewareError(reqwest_middleware::Error, String);

impl From<reqwest_middleware::Error> for Box<dyn CustomErrorTrait> {
    fn from(error: reqwest_middleware::Error) -> Self {
        Box::new(ReqwestMiddlewareError(
            error,
            std::backtrace::Backtrace::capture().to_string(),
        ))
    }
}

impl std::fmt::Display for ReqwestMiddlewareError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ReqwestMiddlewareError {}

impl WithBacktrace for ReqwestMiddlewareError {
    fn backtrace(&self) -> String {
        self.1.clone()
    }
}

impl CustomErrorTrait for ReqwestMiddlewareError {}

/////////////////////////////////////////////

#[derive(Debug)]
struct ReqwestError(reqwest::Error, String);

impl From<reqwest::Error> for Box<dyn CustomErrorTrait> {
    fn from(error: reqwest::Error) -> Self {
        Box::new(ReqwestError(
            error,
            std::backtrace::Backtrace::capture().to_string(),
        ))
    }
}

impl std::fmt::Display for ReqwestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ReqwestError {}

impl WithBacktrace for ReqwestError {
    fn backtrace(&self) -> String {
        self.1.clone()
    }
}

impl CustomErrorTrait for ReqwestError {}

/////////////////////////////////////////////

#[derive(Debug)]
struct SerdeJsonError(serde_json::Error, String);

impl From<serde_json::Error> for Box<dyn CustomErrorTrait> {
    fn from(error: serde_json::Error) -> Self {
        Box::new(SerdeJsonError(
            error,
            std::backtrace::Backtrace::capture().to_string(),
        ))
    }
}

impl std::fmt::Display for SerdeJsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for SerdeJsonError {}

impl WithBacktrace for SerdeJsonError {
    fn backtrace(&self) -> String {
        self.1.clone()
    }
}

impl CustomErrorTrait for SerdeJsonError {}
