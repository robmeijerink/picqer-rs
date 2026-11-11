//! Makes a location, links a product to it, changes it and cleans up.
//!
//! Writes to the account. Pass the id of a product.
//!
//! ```sh
//! cargo run --example locations -- <product id>
//! ```

mod common;

use picqer_rs::locations::{LocationChanges, LocationName, LocationQuery, NewLocation};
use picqer_rs::products::{NewProductLocation, ProductLocationChanges};
use picqer_rs::types::ProductId;

#[tokio::main]
async fn main() -> picqer_rs::Result<()> {
    let id: u64 = std::env::args()
        .nth(1)
        .and_then(|id| id.parse().ok())
        .expect("a product id");
    let client = common::client();

    let warehouse = client.warehouses().list_all().await?[0].idwarehouse;
    let types = client.location_types().list_all().await?;
    println!("{} location type(s)", types.len());

    let locations = client.locations();
    let made = locations
        .create(&NewLocation::new(
            LocationName::new("picqer-rs-example")?,
            warehouse,
        ))
        .await?;
    println!("made {} ({})", made.name, made.idlocation);

    let found = locations
        .list_all(&LocationQuery::new().name("picqer-rs-example"))
        .await?;
    println!("{} location(s) with that name", found.len());

    let renamed = locations
        .update(
            made.idlocation,
            &LocationChanges::new().remarks("made by the picqer-rs example"),
        )
        .await?;
    println!("{:?}", renamed.remarks);

    let product = ProductId(id);
    let products = client.products();
    let linked = products
        .link_location(
            product,
            &NewProductLocation::new(made.idlocation).note("picks fast"),
        )
        .await?;
    println!("{linked:?}");
    products
        .update_location(
            product,
            made.idlocation,
            &ProductLocationChanges::new().preferred(),
        )
        .await?;
    products.unlink_location(product, made.idlocation).await?;

    locations.delete(made.idlocation).await?;

    Ok(())
}
