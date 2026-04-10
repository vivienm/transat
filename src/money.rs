use std::{
    fmt::{self, Display},
    marker::PhantomData,
};

use derive_where::derive_where;
use jiff::civil::Date;
use rust_decimal::Decimal;

/// A currency identified by a static label.
pub trait Currency {
    const LABEL: &'static str;
}

/// The euro (EUR).
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Eur {}

impl Currency for Eur {
    const LABEL: &'static str = "EUR";
}

/// The US dollar (USD).
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Usd {}

impl Currency for Usd {
    const LABEL: &'static str = "USD";
}

/// A monetary amount in a given currency.
#[derive_where(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Amount<C> {
    value: Decimal,
    _phantom: PhantomData<C>,
}

impl<C> Amount<C> {
    pub fn new(value: Decimal) -> Self {
        Self {
            value,
            _phantom: PhantomData,
        }
    }

    pub fn value(&self) -> Decimal {
        self.value
    }

    /// Converts this amount using the given exchange rate.
    pub fn convert<D>(self, rate: &Rate<C, D>) -> Amount<D> {
        Amount::new(self.value * rate.value())
    }
}

impl<C> From<Decimal> for Amount<C> {
    fn from(value: Decimal) -> Self {
        Self::new(value)
    }
}

impl<C> From<Amount<C>> for Decimal {
    fn from(amount: Amount<C>) -> Self {
        amount.value
    }
}

impl<C> Display for Amount<C>
where
    C: Currency,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let precision = f.precision().unwrap_or(DEFAULT_PRECISION);
        write!(f, "{:.prec$} {}", self.value, C::LABEL, prec = precision)
    }
}

/// An exchange rate observed on a given date.
#[derive_where(Clone, Eq, PartialEq, Debug)]
pub struct Rate<B, Q> {
    date: Date,
    value: Decimal,
    _phantom: PhantomData<(B, Q)>,
}

impl<B, Q> Rate<B, Q> {
    pub fn new(date: Date, value: Decimal) -> Self {
        Self {
            date,
            value,
            _phantom: PhantomData,
        }
    }

    pub fn date(&self) -> Date {
        self.date
    }

    pub fn value(&self) -> Decimal {
        self.value
    }

    /// Returns the inverse rate (e.g. EUR/USD -> USD/EUR).
    pub fn invert(&self) -> Rate<Q, B> {
        Rate::new(self.date, Decimal::ONE / self.value)
    }
}

impl<B, Q> Display for Rate<B, Q>
where
    B: Currency,
    Q: Currency,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let precision = f.precision().unwrap_or(DEFAULT_PRECISION);
        write!(
            f,
            "{}/{} = {:.prec$}",
            B::LABEL,
            Q::LABEL,
            self.value,
            prec = precision
        )
    }
}

const DEFAULT_PRECISION: usize = 4;
