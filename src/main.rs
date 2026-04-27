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
    #[arg(required = true)]
    amount: Option<Decimal>,
    /// The source currency.
    #[arg(required = true, ignore_case = true)]
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
    jiff::Zoned::now().date().checked_sub(span)
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

    let today = jiff::Zoned::now().date();
    let date = args.date.unwrap_or(today);
    anyhow::ensure!(date <= today, "date {date} is in the future");

    let value = args.amount.unwrap();
    let currency = args.currency.unwrap();

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

    match currency {
        CurrencyArg::Eur => print!("{}", Amount::<Eur>::new(value).convert(&rate)),
        CurrencyArg::Usd => print!("{}", Amount::<Usd>::new(value).convert(&rate.invert())),
    };
    println!(" ({} on {})", rate, rate.date());

    Ok(())
}
