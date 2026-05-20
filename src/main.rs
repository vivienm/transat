mod ecb;
mod money;

use std::str::FromStr;

use jiff::{Span, civil::Date};
use rust_decimal::Decimal;

use crate::{
    ecb::{EurUsd, ExrRequest},
    money::{Amount, Eur, Usd},
};

#[derive(Debug, clap::Parser)]
#[clap(about)]
struct Args {
    /// Generate the completion script for the specified shell.
    #[arg(long, exclusive = true, name = "SHELL")]
    completion: Option<clap_complete::Shell>,
    /// The conversion date; the latest published rate on or before this date is used.
    #[arg(short, long)]
    date: Option<DateArg>,
    /// How far back to look for a rate if none is published on the target date.
    #[arg(short, long, value_parser = parse_lookback, default_value = "7 days")]
    lookback: Span,
    /// The amount to convert.
    #[arg(requires = "currency")]
    amount: Option<Decimal>,
    /// The source currency.
    #[arg(requires = "amount", ignore_case = true)]
    currency: Option<CurrencyArg>,
}

/// A date argument: either absolute, or relative to "today".
#[derive(Clone, Debug)]
enum DateArg {
    Absolute(Date),
    Ago(Span),
}

impl DateArg {
    fn resolve(&self, today: Date) -> Result<Date, jiff::Error> {
        match self {
            Self::Absolute(d) => Ok(*d),
            Self::Ago(span) => today.checked_sub(*span),
        }
    }
}

impl FromStr for DateArg {
    type Err = jiff::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(date) = s.parse::<Date>() {
            return Ok(Self::Absolute(date));
        }
        Ok(Self::Ago(s.parse::<Span>()?.abs()))
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, clap::ValueEnum)]
enum CurrencyArg {
    Eur,
    Usd,
}

fn generate_completions(shell: clap_complete::Shell) {
    clap_complete::generate(
        shell,
        &mut <Args as clap::CommandFactory>::command(),
        clap::crate_name!(),
        &mut std::io::stdout(),
    );
}

fn parse_lookback(s: &str) -> anyhow::Result<Span> {
    // The reference date is only needed so jiff can resolve calendar units
    // (months/years) into days; any fixed date works for our validation.
    let relative = Date::constant(2000, 1, 1);
    let span = s.parse::<Span>()?.abs();
    let days = span.total((jiff::Unit::Day, relative))?;
    if days.fract() != 0.0 {
        anyhow::bail!("lookback must be a whole number of days");
    }
    Ok(span)
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let args = <Args as clap::Parser>::parse();
    if let Some(shell) = args.completion {
        generate_completions(shell);
        return Ok(());
    }

    let today = ecb::today()?;
    let date = args
        .date
        .map(|d| d.resolve(today))
        .transpose()?
        .unwrap_or(today);
    anyhow::ensure!(date <= today, "date {date} is in the future");

    let client: ecb::Client = ecb::Client::new();
    let response = client
        .execute(ExrRequest::new(
            EurUsd::Daily,
            date.saturating_sub(args.lookback),
            date,
        ))
        .await?;

    let rate = response
        .find_rate(date)
        .ok_or_else(|| anyhow::anyhow!("no exchange rate available for {date}"))?;

    if let Some((value, currency)) = args.amount.zip(args.currency) {
        match currency {
            CurrencyArg::Eur => {
                println!("{} ({})", Amount::<Eur>::new(value).convert(&rate), rate);
            }
            CurrencyArg::Usd => {
                println!(
                    "{} ({})",
                    Amount::<Usd>::new(value).convert(&rate.invert()),
                    rate,
                );
            }
        }
    } else {
        println!("{rate}");
    }

    Ok(())
}
