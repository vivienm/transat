use std::sync::Arc;

use tower::{Service, ServiceExt};
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
        S::default().into()
    }
}

impl<S> Client<S> {
    pub fn builder(service: S) -> ClientBuilder<S> {
        ClientBuilder::new(service)
    }

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

impl<S> From<S> for Client<S> {
    fn from(service: S) -> Self {
        Self::builder(service).build()
    }
}

#[derive(Debug)]
pub struct ClientBuilder<S = reqwest::Client> {
    service: S,
    base_url: Option<Arc<Url>>,
}

impl<S> ClientBuilder<S> {
    pub fn new(service: S) -> Self {
        Self {
            service,
            base_url: None,
        }
    }

    #[expect(dead_code)]
    pub fn with_url<U>(mut self, url: U) -> Self
    where
        U: Into<Arc<Url>>,
    {
        self.base_url = Some(url.into());
        self
    }

    pub fn build(self) -> Client<S> {
        let base_url = self.base_url.unwrap_or_else(|| {
            Arc::new(Url::parse(DEFAULT_BASE_URL).expect("invalid default base URL"))
        });
        Client {
            service: self.service,
            base_url,
        }
    }
}

impl<S> From<S> for ClientBuilder<S> {
    fn from(service: S) -> Self {
        ClientBuilder::new(service)
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
