//! The same calls without `async`, with the `blocking` feature.
//!
//! ```sh
//! cargo run --example blocking --features blocking
//! ```

use picqer_rs::blocking::Client;
use picqer_rs::pagination::Page;
use picqer_rs::products::ProductQuery;
use picqer_rs::{ApiKey, Subdomain, UserAgent};

fn main() -> picqer_rs::Result<()> {
    let user_agent = UserAgent::new("picqer-rs examples", "dev@example.com")?;
    let client = Client::new(Subdomain::from_env()?, ApiKey::from_env()?, user_agent)?;

    for product in client
        .products()
        .list(&ProductQuery::new(), Page::FIRST)?
        .iter()
        .take(5)
    {
        println!("{} {}", product.productcode, product.name);
    }

    Ok(())
}
