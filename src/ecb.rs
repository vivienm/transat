#![expect(unused_imports)]

mod client;
mod endpoints;
mod models;

pub use self::{
    client::{Client, ClientError, ClientLayer},
    endpoints::{EurUsd, ExrRequest, ExrResponse},
};

/// Returns "today" in the ECB's timezone.
///
/// Anchoring on CET/CEST avoids users west of CET missing the latest
/// published rate, and users east of it requesting a date that hasn't
/// been reached at the ECB yet.
pub fn today() -> Result<jiff::civil::Date, jiff::Error> {
    Ok(jiff::Zoned::now().in_tz("Europe/Berlin")?.date())
}
