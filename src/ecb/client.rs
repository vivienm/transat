use std::sync::Arc;

use tower::{Layer, Service, ServiceExt};
use url::Url;

const DEFAULT_BASE_URL: &str = "https://data-api.ecb.europa.eu";

#[derive(Clone, Debug)]
pub struct Client<S = reqwest::Client> {
    pub(super) service: S,
    pub(super) base_url: Arc<Url>,
}

impl<S> Client<S>
where
    S: Default,
{
    pub fn new() -> Self {
        ClientLayer::default().layer(S::default())
    }
}

impl<S> Client<S> {
    pub async fn execute<R>(
        &self,
        request: R,
    ) -> Result<<Client<S> as Service<R>>::Response, <Client<S> as Service<R>>::Error>
    where
        S: Clone,
        Client<S>: Service<R>,
    {
        self.clone().oneshot(request).await
    }
}

#[derive(Clone, Debug, Default)]
pub struct ClientLayer {
    base_url: Option<Arc<Url>>,
}

impl ClientLayer {
    #[expect(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    #[expect(dead_code)]
    pub fn with_url<U>(mut self, url: U) -> Self
    where
        U: Into<Arc<Url>>,
    {
        self.base_url = Some(url.into());
        self
    }
}

impl<S> Layer<S> for ClientLayer {
    type Service = Client<S>;

    fn layer(&self, service: S) -> Self::Service {
        let base_url = match &self.base_url {
            Some(url) => Arc::clone(url),
            None => Arc::new(Url::parse(DEFAULT_BASE_URL).expect("invalid default URL")),
        };
        Client { service, base_url }
    }
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum ClientError<S = reqwest::Client>
where
    S: Service<reqwest::Request>,
{
    Url(url::ParseError),
    Service(S::Error),
    Response(reqwest::Error),
}
