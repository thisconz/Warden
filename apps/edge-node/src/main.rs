use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;

use proxy::{ProxyBody, ProxyEngine};

use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap();

    println!("warden edge-node listening on {}", addr);

    let proxy = Arc::new(
        ProxyEngine::new("http://httpbin.org")
    );

    loop {
        let (stream, _) = listener.accept().await.unwrap();

        let io = TokioIo::new(stream);

        let proxy = proxy.clone();

        tokio::spawn(async move {
            let service = service_fn(move |req: Request<Incoming>| {
                let proxy = proxy.clone();

                async move {
                    handle_request(req, proxy).await
                }
            });

            if let Err(err) = Builder::new(TokioExecutor::new())
                .serve_connection(io, service)
                .await
            {
                eprintln!("connection error: {:?}", err);
            }
        });
    }
}

async fn handle_request(
    req: Request<Incoming>,
    proxy: Arc<ProxyEngine>,
) -> Result<Response<Incoming>, Infallible> {
    let start = Instant::now();

    let method = req.method().to_string();
    let path = req.uri().path().to_string();

    let body_bytes = req
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();

    let proxied_request = Request::builder()
        .method(method.as_str())
        .uri(path.as_str())
        .body(Full::new(Bytes::from(body_bytes)))
        .unwrap();

    match proxy.forward(proxied_request).await {
        Ok(response) => {
            telemetry::log_request(
                &method,
                &path,
                response.status().as_u16(),
                start.elapsed(),
            );

            Ok(response)
        }

        Err(_) => {
            let response = Response::builder()
                .status(502)
                .body(
                    Incoming::empty()
                )
                .unwrap();

            Ok(response)
        }
    }
}