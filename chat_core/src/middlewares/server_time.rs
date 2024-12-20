use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use axum::{extract::Request, response::Response};
use tower::{Layer, Service};
use tracing::warn;

use crate::middlewares::REQUEST_ID_HEADER;

use super::SERVER_TIME_HEADER;

#[derive(Clone)]
pub struct ServerTime;

impl<S> Layer<S> for ServerTime {
    type Service = ServerTimeMiddleWare<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ServerTimeMiddleWare { inner }
    }
}

#[derive(Clone)]
pub struct ServerTimeMiddleWare<S> {
    inner: S,
}

impl<S> Service<Request> for ServerTimeMiddleWare<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    // `BoxFuture` is a type alias for `Pin<Box<dyn Future + Send + 'a>>`
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let start = tokio::time::Instant::now();
        let future = self.inner.call(request);
        Box::pin(async move {
            let mut response: Response = future.await?;
            let elapsed = format!("{}us", start.elapsed().as_micros());

            match elapsed.parse() {
                Ok(v) => {
                    response.headers_mut().insert(SERVER_TIME_HEADER, v);
                }
                Err(e) => {
                    warn!(
                        "Parse elapsed time failed: {} for request {:?}",
                        e,
                        response.headers().get(REQUEST_ID_HEADER)
                    );
                }
            }

            Ok(response)
        })
    }
}
