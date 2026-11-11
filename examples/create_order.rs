//! Makes an order for a customer, processes it and waits until Picqer is done.
//!
//! Writes to the account. Pass the id of a customer and of a product that belong together (the
//! same fulfilment customer, if you use those).
//!
//! ```sh
//! cargo run --example create_order -- <customer id> <product id>
//! ```

mod common;

use std::num::NonZeroU32;
use std::time::Duration;

use picqer_rs::orders::{NewOrder, NewOrderProduct, OrderStatus};
use picqer_rs::types::{CustomerId, NonEmpty, OrderId, ProductId};
use picqer_rs::{Client, Reported};

#[tokio::main]
async fn main() -> picqer_rs::Result<()> {
    let mut args = std::env::args().skip(1);
    let customer: u64 = args
        .next()
        .and_then(|id| id.parse().ok())
        .expect("a customer id");
    let product: u64 = args
        .next()
        .and_then(|id| id.parse().ok())
        .expect("a product id");

    let client = common::client();

    let line = NewOrderProduct::by_id(ProductId(product), NonZeroU32::new(1).unwrap());
    let order = NewOrder::for_customer(CustomerId(customer), NonEmpty::new(vec![line])?)
        .reference("picqer-rs example");
    let created = client.orders().create(&order).await?;
    println!("{} is {:?}", created.orderid, created.status);

    client.orders().process(created.idorder).await?;
    let processed = wait_for(&client, created.idorder, OrderStatus::Processing).await?;
    println!("{} is now {:?}", processed.orderid, processed.status);
    println!("it has {} picklist(s)", processed.picklists.len());

    Ok(())
}

/// Picqer changes the status in the background, so the answer of `process` still shows the old one.
async fn wait_for(
    client: &Client,
    id: OrderId,
    wanted: OrderStatus,
) -> picqer_rs::Result<picqer_rs::orders::Order> {
    for _ in 0..30 {
        let order = client.orders().get(id).await?;
        if order.status == Reported::Known(wanted) {
            return Ok(order);
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    client.orders().get(id).await
}
