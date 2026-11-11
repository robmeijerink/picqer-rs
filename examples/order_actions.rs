//! Changes an order and moves it through its statuses: lines, tags, allocation, pause and cancel.
//!
//! Writes to the account. It makes its own order, so pass a customer and a product that belong
//! together. Picqer does the work of a status change in the background, so a call right after
//! another can answer `ErrorCode::OrderBeingProcessed`: try again a little later.
//!
//! ```sh
//! cargo run --example order_actions -- <customer id> <product id>
//! ```

mod common;

use std::num::NonZeroU32;

use picqer_rs::orders::{NewOrder, NewOrderLine, NewOrderProduct, OrderChanges};
use picqer_rs::pagination::Page;
use picqer_rs::types::{CustomerId, NonEmpty, ProductId};

#[tokio::main]
async fn main() -> picqer_rs::Result<()> {
    let mut args = std::env::args()
        .skip(1)
        .map(|id| id.parse::<u64>().expect("an id"));
    let customer = CustomerId(args.next().expect("a customer id"));
    let product = ProductId(args.next().expect("a product id"));

    let client = common::client();
    let orders = client.orders();
    let one = NonZeroU32::new(1).unwrap();

    let line = NewOrderProduct::by_id(product, one);
    let order = NewOrder::for_customer(customer, NonEmpty::new(vec![line])?);
    let order = orders.create(&order).await?;
    let id = order.idorder;

    orders
        .update(id, &OrderChanges::new().reference("picqer-rs example"))
        .await?;
    orders
        .add_line(id, &NewOrderLine::new(product, one))
        .await?;
    for line in orders.lines(id).await? {
        println!("{} x {:?}", line.amount, line.productcode);
    }
    println!("{:?}", orders.product_status(id).await?);

    if let Some(tag) = client.tags().list_all(&Default::default()).await?.first() {
        orders.add_tag(id, tag.idtag).await?;
        println!("{} tag(s)", orders.tags(id, Page::FIRST).await?.len());
        orders.remove_tag(id, tag.idtag).await?;
    }

    orders.prioritise(id).await?;
    orders.allocate(id).await?;
    orders.deallocate(id).await?;

    orders.cancel(id).await?;
    orders.undo_cancellation(id).await?;
    orders.cancel(id).await?;

    Ok(())
}
