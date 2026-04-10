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

fn generate_completions(shell: clap_complete::Shell) -> ! {
    clap_complete::generate(
        shell,
        &mut <Args as clap::CommandFactory>::command(),
        clap::crate_name!(),
        &mut std::io::stdout(),
    );
    std::process::exit(0);
}

fn parse_date(s: &str) -> Result<Date, jiff::Error> {
    if let Ok(date) = s.parse::<Date>() {
        return Ok(date);
    }
    let span = s.parse::<jiff::Span>()?.abs();
    jiff::Zoned::now().date().checked_sub(span)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = <Args as clap::Parser>::parse();
    if let Some(shell) = args.completion {
        generate_completions(shell);
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
            date.saturating_sub(Span::new().days(7)),
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
