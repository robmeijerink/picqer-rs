//! Lists the customers of Picqer Fulfilment and reads the report of one.
//!
//! Reads only: Picqer cannot delete a fulfilment customer, so making one is left to you with
//! `NewFulfilmentCustomer::new(name, Language::Nl)`.
//!
//! ```sh
//! cargo run --example fulfilment_customers
//! ```

mod common;

use picqer_rs::fulfilment::ReportPeriod;

#[tokio::main]
async fn main() -> picqer_rs::Result<()> {
    let client = common::client();
    let customers = client.fulfilment_customers().list_all().await?;
    println!("{} fulfilment customer(s)", customers.len());

    if let Some(customer) = customers.first() {
        let report = client
            .fulfilment_customers()
            .report(customer.idfulfilment_customer, ReportPeriod::new())
            .await?;
        println!("{report:?}");
    }

    Ok(())
}
