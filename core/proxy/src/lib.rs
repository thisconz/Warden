use bytes::Bytes;
use http_body_util::Full;
use hyper::{Request, Response, Uri};
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::TokioExecutor;

pub type ProxyBody = Full<Bytes>;

#[derive(Clone)]
pub struct ProxyEngine {
    client: Client<HttpConnector, ProxyBody>,
    origin: String,
}

impl ProxyEngine {
    pub fn new(origin: impl Into<String>) -> Self {
        let connector = HttpConnector::new();

        let client = Client::builder(TokioExecutor::new())
            .build(connector);

        Self {
            client,
            origin: origin.into(),
        }
    }

    pub async fn forward(
        &self,
        req: Request<ProxyBody>,
    ) -> Result<Response<hyper::body::Incoming>, hyper::Error> {
        let path = req
            .uri()
            .path_and_query()
            .map(|p| p.as_str())
            .unwrap_or("/");

        let uri: Uri = format!("{}{}", self.origin, path)
            .parse()
            .expect("invalid origin uri");

        let (mut parts, body) = req.into_parts();
        parts.uri = uri;

        let proxied_request = Request::from_parts(parts, body);

        self.client.request(proxied_request).await
    }
}