//! Handles errors by their code and deals with the rate limit. Reads only.
//!
//! ```sh
//! cargo run --example errors_and_rate_limit
//! ```

mod common;

use std::num::NonZeroU32;

use picqer_rs::types::OrderId;
use picqer_rs::{ApiKey, Client, Error, ErrorCode, Reported, Subdomain, UserAgent};

#[tokio::main]
async fn main() -> picqer_rs::Result<()> {
    let client = common::client();

    match client.orders().get(OrderId(1)).await {
        Ok(order) => println!("found {}", order.orderid),
        Err(error) if error.is_not_found() => println!("no such order"),
        Err(Error::RateLimited { resets_in }) => println!("try again in {resets_in:?}"),
        Err(Error::Api(error)) => match error.code {
            Some(Reported::Known(ErrorCode::OrderBeingProcessed)) => println!("busy, try again"),
            _ => println!("{} {}", error.status, error.message),
        },
        Err(error) => return Err(error),
    }

    if let Some(limit) = client.rate_limit() {
        println!(
            "{} of {} requests left, the count restarts in {:?}",
            limit.remaining(),
            limit.limit(),
            limit.resets_in()
        );
    }

    let patient = Client::builder(
        Subdomain::from_env()?,
        ApiKey::from_env()?,
        UserAgent::new("picqer-rs examples", "dev@example.com")?,
    )
    .retry_on_rate_limit(NonZeroU32::new(2).unwrap())
    .throttle()
    .build()?;
    println!(
        "{} warehouses",
        patient.warehouses().list_all().await?.len()
    );

    Ok(())
}
