mod ecb;
mod money;

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
    /// The date for which to fetch the exchange rate.
    #[arg(short, long, value_parser = parse_date)]
    date: Option<Date>,
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

fn parse_date(s: &str) -> Result<Date, jiff::Error> {
    if let Ok(date) = s.parse::<Date>() {
        return Ok(date);
    }
    let span = s.parse::<jiff::Span>()?.abs();
    jiff::Zoned::now().in_tz("Europe/Berlin")?.date().checked_sub(span)
}

fn parse_lookback(s: &str) -> anyhow::Result<Span> {
    let relative = jiff::Zoned::now();
    let span = s.parse::<Span>()?.abs();
    let days = span.total((jiff::Unit::Day, &relative))?;
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

    // Anchor "today" on the ECB's timezone so users west of CET don't miss
    // the latest published rate, and users east don't request a date that
    // hasn't been reached at the ECB yet.
    let today = jiff::Zoned::now().in_tz("Europe/Berlin")?.date();
    let date = args.date.unwrap_or(today);
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

    match (args.amount, args.currency) {
        (Some(value), Some(CurrencyArg::Eur)) => {
            println!("{} ({})", Amount::<Eur>::new(value).convert(&rate), rate);
        }
        (Some(value), Some(CurrencyArg::Usd)) => {
            println!(
                "{} ({})",
                Amount::<Usd>::new(value).convert(&rate.invert()),
                rate,
            );
        }
        (None, None) => println!("{}", rate),
        _ => unreachable!("clap enforces amount and currency together"),
    }

    Ok(())
}
