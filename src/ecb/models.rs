use std::collections::BTreeMap;

use jiff::civil::DateTime;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExrResponse {
    pub data_sets: [ExrResponseDataset; 1],
    pub structure: ExrResponseStructure,
}

#[derive(Debug, Deserialize)]
pub struct ExrResponseDataset {
    pub series: BTreeMap<String, ExrDatasetSeries>,
}

#[derive(Debug, Deserialize)]
pub struct ExrDatasetSeries {
    pub observations: BTreeMap<usize, ExrSeriesObservation>,
}

#[derive(Debug, Deserialize)]
pub struct ExrSeriesObservation(
    Decimal,
    #[expect(dead_code)] u8,         // Status.
    #[expect(dead_code)] u8,         // Confidentiality.
    #[expect(dead_code)] Option<u8>, // Pre-break.
    #[expect(dead_code)] Option<u8>, // Comment.
);

impl ExrSeriesObservation {
    pub fn rate(&self) -> Decimal {
        self.0
    }
}

#[derive(Debug, Deserialize)]
pub struct ExrResponseStructure {
    pub dimensions: ExrStructureDimension,
}

#[derive(Debug, Deserialize)]
pub struct ExrStructureDimension {
    pub observation: [ExrDimensionObservation; 1],
}

#[derive(Debug, Deserialize)]
pub struct ExrDimensionObservation {
    #[expect(dead_code)]
    pub id: ExrObservationId,
    pub values: Vec<ExrObservationValue>,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExrObservationId {
    TimePeriod,
}

#[derive(Debug, Deserialize)]
pub struct ExrObservationValue {
    pub start: DateTime,
    #[expect(dead_code)]
    pub end: DateTime,
}
