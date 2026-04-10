# Transat

A simple CLI for euro-dollar conversions based on official [European Central Bank](https://www.ecb.europa.eu/) (ECB) exchange rates.

💵✨💶

## Installation

```sh
cargo install --git https://github.com/vivienm/transat
```

## Usage

Convert 100 EUR to USD using today's rate:

```sh
transat 100 eur
```

Convert 50 USD to EUR using the rate from 3 days ago:

```sh
transat 50 usd -d '3 days ago'
```

Convert using a specific date:

```sh
transat 100 eur -d 2025-12-01
```
