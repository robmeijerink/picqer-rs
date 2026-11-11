//! Lists orders, with a filter and with pagination. Reads only.
//!
//! ```sh
//! PICQER_SUBDOMAIN=example PICQER_API_KEY=... cargo run --example list_orders
//! ```

mod common;

use picqer_rs::orders::{OrderQuery, OrderStatus};
use picqer_rs::pagination::Page;

#[tokio::main]
async fn main() -> picqer_rs::Result<()> {
    let client = common::client();

    let query = OrderQuery::new().status(OrderStatus::Processing);
    for order in client
        .orders()
        .list(&query, Page::FIRST)
        .await?
        .iter()
        .take(5)
    {
        println!(
            "{} {}",
            order.orderid,
            order.deliveryname.as_deref().unwrap_or("-")
        );
    }

    let second_page = client.orders().list(&query, Page::new(1)).await?;
    println!("page 2 holds {} orders", second_page.len());

    let all = client.orders().list_all(&query).await?;
    println!("{} orders are being processed", all.len());

    Ok(())
}
