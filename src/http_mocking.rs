// use http::Extensions;
// use reqwest::{Client, Request, Response};
// use reqwest_middleware::{ClientBuilder, Middleware, Next, Result};

// struct LoggingMiddleware;

// #[async_trait::async_trait]
// impl Middleware for LoggingMiddleware {
//     async fn handle(
//         &self,
//         req: Request,
//         extensions: &mut Extensions,
//         next: Next<'_>,
//     ) -> Result<Response> {
//         println!("Request started {:?}", req);
//         let res = next.run(req, extensions).await;
//         println!("Result: {:?}", res);
//         res
//     }
// }
