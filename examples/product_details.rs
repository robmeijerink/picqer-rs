//! Reads the history and the warehouse settings of a product and changes its settings.
//!
//! Writes to the account, and puts the order and desired levels back. Pass the id of a product.
//!
//! ```sh
//! cargo run --example product_details -- <product id>
//! ```

mod common;

use picqer_rs::products::WarehouseSettingsChanges;
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

    println!(
        "{} price change(s)",
        products.price_history(product).await?.len()
    );
    println!(
        "{} fixed stock price change(s)",
        products.fixed_stock_price_history(product).await?.len()
    );
    println!(
        "{} expected delivery(ies)",
        products.expected(product).await?.len()
    );

    let warehouse = client.warehouses().list_all().await?[0].idwarehouse;
    let before = products.warehouse_settings(product).await?;
    println!("{} warehouse setting(s)", before.len());

    let changes = WarehouseSettingsChanges::new()
        .stock_level_order(5)
        .stock_level_desired(10);
    let changed = products
        .update_warehouse_settings(product, warehouse, &changes)
        .await?;
    println!("{changed:?}");
    let original = before
        .iter()
        .find(|settings| settings.idwarehouse == warehouse);
    let level = |value: Option<i64>| u32::try_from(value.unwrap_or(0)).unwrap_or(0);
    let reset = WarehouseSettingsChanges::new()
        .stock_level_order(level(
            original.and_then(|settings| settings.stock_level_order),
        ))
        .stock_level_desired(level(
            original.and_then(|settings| settings.stock_level_desired),
        ));
    products
        .update_warehouse_settings(product, warehouse, &reset)
        .await?;

    Ok(())
}
