//! Reads the fields made in the Picqer app and sets a value on a product, an order and a customer.
//!
//! Writes to the account. Fields are made in the app, so the example uses the first field of
//! each kind that exists. Picqer stores any text, so `value` and `choices` check a value against
//! the choices of the field before it is sent.
//!
//! ```sh
//! cargo run --example custom_fields -- <product id> <order id> <customer id>
//! ```

mod common;

use picqer_rs::products::ProductChanges;
use picqer_rs::types::{CustomerId, OrderId, ProductId};

#[tokio::main]
async fn main() -> picqer_rs::Result<()> {
    let mut args = std::env::args()
        .skip(1)
        .map(|id| id.parse::<u64>().expect("an id"));
    let product = ProductId(args.next().expect("a product id"));
    let order = OrderId(args.next().expect("an order id"));
    let customer = CustomerId(args.next().expect("a customer id"));

    let client = common::client();

    if let Some(field) = client.product_fields().list_all().await?.first() {
        println!("product field {:?} is {:?}", field.title, field.field_type);
        let value = field.value("example")?;
        let changes = ProductChanges::new().productfields(vec![value]);
        client.products().update(product, &changes).await?;
        client
            .products()
            .update(
                product,
                &ProductChanges::new().productfields(vec![field.cleared()]),
            )
            .await?;
    }

    if let Some(field) = client.order_fields().list_all().await?.first() {
        println!("order field {:?} is {:?}", field.title, field.field_type);
        client
            .orders()
            .set_field(order, &field.value("example")?)
            .await?;
        println!("{:?}", client.orders().fields(order).await?);
        client
            .orders()
            .remove_field(order, field.idorderfield)
            .await?;
    }

    if let Some(field) = client.customer_fields().list_all().await?.first() {
        println!("customer field {:?} is {:?}", field.title, field.field_type);
        let changes = picqer_rs::customers::CustomerChanges::new()
            .customerfields(vec![field.value("example")?]);
        client.customers().update(customer, &changes).await?;
        let cleared =
            picqer_rs::customers::CustomerChanges::new().customerfields(vec![field.cleared()]);
        client.customers().update(customer, &cleared).await?;
    }

    Ok(())
}
