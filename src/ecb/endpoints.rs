use std::{marker::PhantomData, pin::Pin, task};

use derive_where::derive_where;
use jiff::civil::Date;
use tower::{Service, ServiceExt};
use url::Url;

use super::{
    client::{Client, ClientError},
    models,
};
use crate::money::{Currency, Eur, Rate, Usd};

/// A request to the ECB exchange rate API for a given dataset and date range.
#[derive(Clone, PartialEq, Debug)]
pub struct ExrRequest<D> {
    dataset: D,
    start: Date,
    end: Date,
}

impl<D> ExrRequest<D> {
    pub fn new(dataset: D, start: Date, end: Date) -> Self {
        Self {
            dataset,
            start,
            end,
        }
    }
}

/// An ECB exchange rate dataset, mapping to a specific API series key.
pub trait ExrDataset {
    type Base: Currency;
    type Quote: Currency;

    fn key(self) -> &'static str;
}

/// The EUR/USD exchange rate dataset.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum EurUsd {
    Daily,
}

impl ExrDataset for EurUsd {
    type Base = Eur;
    type Quote = Usd;

    fn key(self) -> &'static str {
        match self {
            Self::Daily => "D.USD.EUR.SP00.A",
        }
    }
}

/// A typed wrapper around the raw ECB exchange rate response.
#[derive_where(Debug)]
pub struct ExrResponse<B, Q> {
    model: models::ExrResponse,
    _phantom: PhantomData<(B, Q)>,
}

impl<B, Q> ExrResponse<B, Q> {
    fn new(model: models::ExrResponse) -> Self {
        Self {
            model,
            _phantom: PhantomData,
        }
    }

    /// Finds the exchange rate for `date`, or the closest preceding date
    /// if unavailable (e.g. weekends or holidays).
    pub fn find_rate(&self, date: Date) -> Option<Rate<B, Q>> {
        let dimension = &self.model.structure.dimensions.observation[0];
        let series = self.model.data_sets[0].series.values().next()?;

        series
            .observations
            .iter()
            .filter_map(|(&idx, obs)| {
                let obs_date = dimension.values.get(idx)?.start.date();
                Some(Rate::new(obs_date, obs.rate()))
            })
            .filter(|rate| rate.date() <= date)
            .max_by_key(|rate| rate.date())
    }
}

impl<S, D> Service<ExrRequest<D>> for Client<S>
where
    S: Service<reqwest::Request, Response = reqwest::Response> + 'static,
    D: ExrDataset,
{
    type Response = ExrResponse<D::Base, D::Quote>;
    type Error = ClientError<S>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut task::Context<'_>) -> task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx).map_err(ClientError::Service)
    }

    fn call(&mut self, req: ExrRequest<D>) -> Self::Future {
        let mut url = Url::clone(&self.base_url);

        let Ok(mut segments) = url.path_segments_mut() else {
            return Box::pin(async {
                Err(ClientError::Url(url::ParseError::RelativeUrlWithoutBase))
            });
        };
        segments.pop_if_empty();
        segments.extend(&["service", "data", "EXR", req.dataset.key()]);
        drop(segments);

        url.query_pairs_mut()
            .append_pair("startPeriod", &req.start.to_string())
            .append_pair("endPeriod", &req.end.to_string())
            .append_pair("format", "jsondata");

        let request = reqwest::Request::new(reqwest::Method::GET, url);
        let response_fut = self.service.call(request);

        Box::pin(async move {
            let response = response_fut
                .await
                .map_err(ClientError::Service)?
                .error_for_status()
                .map_err(ClientError::Response)?
                .json()
                .await
                .map(ExrResponse::new)
                .map_err(ClientError::Response)?;

            Ok(response)
        })
    }
}

impl<S, D> Service<ExrRequest<D>> for &Client<S>
where
    S: Service<reqwest::Request, Response = reqwest::Response> + Clone + 'static,
    D: ExrDataset,
{
    type Response = ExrResponse<D::Base, D::Quote>;
    type Error = ClientError<S>;
    type Future = tower::util::Oneshot<Client<S>, ExrRequest<D>>;

    fn poll_ready(&mut self, _cx: &mut task::Context<'_>) -> task::Poll<Result<(), Self::Error>> {
        task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: ExrRequest<D>) -> Self::Future {
        Client::clone(self).oneshot(req)
    }
}
