//! Orders a product from a supplier, receives it and cleans up.
//!
//! Writes to the account. Pass the id of a product. The receipt is deleted again and the purchase
//! order is cancelled, but Picqer cannot delete a supplier or a purchase order, so the supplier
//! `picqer-rs example` stays and is reused.
//!
//! ```sh
//! cargo run --example purchasing -- <product id>
//! ```

mod common;

use std::num::NonZeroU32;

use picqer_rs::purchase_orders::{NewPurchaseOrder, NewPurchaseOrderProduct};
use picqer_rs::receipts::{NewReceipt, ReceiveProduct, ReceiveStrategy};
use picqer_rs::suppliers::{NewSupplier, SupplierName};
use picqer_rs::types::ProductId;

#[tokio::main]
async fn main() -> picqer_rs::Result<()> {
    let id: u64 = std::env::args()
        .nth(1)
        .and_then(|id| id.parse().ok())
        .expect("a product id");
    let product = ProductId(id);
    let client = common::client();

    let name = SupplierName::new("picqer-rs example")?;
    let existing = client.suppliers().list_all().await?;
    let supplier = match existing
        .into_iter()
        .find(|supplier| supplier.name == "picqer-rs example")
    {
        Some(supplier) => supplier,
        None => client.suppliers().create(&NewSupplier::new(name)).await?,
    };

    let five = NonZeroU32::new(5).unwrap();
    let order = NewPurchaseOrder::for_supplier(supplier.idsupplier)
        .supplier_orderid("picqer-rs-example")
        .product(NewPurchaseOrderProduct::new(product, five));
    let order = client.purchase_orders().create(&order).await?;
    println!(
        "purchase order {} is {:?}",
        order.purchaseorderid, order.status
    );
    client
        .purchase_orders()
        .mark_as_purchased(order.idpurchaseorder)
        .await?;

    let receipt = client
        .receipts()
        .create(NewReceipt::for_purchase_order(order.idpurchaseorder))
        .await?;
    for expected in client
        .receipts()
        .expected_products(receipt.idreceipt)
        .await?
    {
        println!("expected: {expected:?}");
    }

    let lines = client
        .receipts()
        .receive(
            receipt.idreceipt,
            &ReceiveProduct::new(product, five, ReceiveStrategy::Automatic),
        )
        .await?;
    for line in &lines {
        client
            .receipts()
            .revert(receipt.idreceipt, line.idreceipt_product)
            .await?;
    }

    client.receipts().delete(receipt.idreceipt).await?;
    client
        .purchase_orders()
        .cancel(order.idpurchaseorder)
        .await?;

    Ok(())
}
