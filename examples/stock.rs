//! Changes and reads the stock of a product, moves stock between places and reads the history.
//!
//! Writes to the account. Pass the id of a product that is in the warehouse.
//!
//! ```sh
//! cargo run --example stock -- <product id>
//! ```

mod common;

use std::num::{NonZeroI64, NonZeroU32};

use picqer_rs::stock::{StockMove, StockPlace, StockReason, StockTarget, StockUpdate};
use picqer_rs::stock_history::StockHistoryQuery;
use picqer_rs::types::ProductId;

#[tokio::main]
async fn main() -> picqer_rs::Result<()> {
    let id: u64 = std::env::args()
        .nth(1)
        .and_then(|id| id.parse().ok())
        .expect("a product id");
    let product = ProductId(id);

    let client = common::client();
    let products = client.products();

    let levels = products.stock(product).await?;
    let warehouse = levels
        .first()
        .expect("the product has stock in a warehouse")
        .idwarehouse;
    for level in &levels {
        println!(
            "warehouse {}: {} in stock, {} free",
            level.idwarehouse, level.stock, level.freestock
        );
    }

    let reason = || StockReason::new("picqer-rs example");
    let add = StockUpdate::adjust(
        StockTarget::NoSpecificLocation,
        NonZeroI64::new(5).unwrap(),
        reason()?,
    );
    products.update_stock(product, warehouse, &add).await?;

    let location = client
        .locations()
        .list_all(&Default::default())
        .await?
        .into_iter()
        .find(|location| location.idwarehouse == warehouse)
        .expect("the warehouse has a location");
    let move_two = StockMove::new(
        NonZeroU32::new(2).unwrap(),
        StockPlace::NoSpecificLocation(warehouse),
        StockPlace::Location(location.idlocation),
    );
    let moved = products.move_stock(product, &move_two).await?;
    println!("moved {} to {}", moved.amount, location.name);

    let detail = products.stock_in_warehouse(product, warehouse).await?;
    println!("{detail:#?}");

    let remove = StockUpdate::adjust(
        StockTarget::NoSpecificLocation,
        NonZeroI64::new(-3).unwrap(),
        reason()?,
    );
    products.update_stock(product, warehouse, &remove).await?;

    let query = StockHistoryQuery::new()
        .idproduct(product)
        .idwarehouse(warehouse);
    for entry in client
        .stock_history()
        .list_all(&query)
        .await?
        .iter()
        .take(5)
    {
        println!("{entry:?}");
    }

    Ok(())
}
