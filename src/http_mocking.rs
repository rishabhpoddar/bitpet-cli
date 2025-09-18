use http::Extensions;
use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next, Result};

pub struct MockingMiddleware;

#[async_trait::async_trait]
impl Middleware for MockingMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        println!("Request started {:?}", req);
        let res = next.run(req, extensions).await;
        println!("Result: {:?}", res);
        res
    }
}
