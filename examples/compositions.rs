//! Gives a product parts, reads the compositions and produces some.
//!
//! Writes to the account. The product must be of the type `virtual_composition` or
//! `composition_with_stock` and the part must belong to the same fulfilment customer.
//!
//! ```sh
//! cargo run --example compositions -- <composition id> <part id>
//! ```

mod common;

use std::num::NonZeroU32;

use picqer_rs::products::{NewProductPart, Production};
use picqer_rs::stock::StockReason;
use picqer_rs::types::ProductId;

#[tokio::main]
async fn main() -> picqer_rs::Result<()> {
    let mut args = std::env::args()
        .skip(1)
        .map(|id| id.parse::<u64>().expect("an id"));
    let composition = ProductId(args.next().expect("a composition id"));
    let part = ProductId(args.next().expect("a part id"));

    let client = common::client();
    let products = client.products();

    let two = NonZeroU32::new(2).unwrap();
    let added = products
        .add_part(composition, &NewProductPart::new(part, two))
        .await?;
    println!("{} needs {} of {}", composition, added.amount, added.name);

    for part in products.parts(composition).await? {
        println!("part {} x {}", part.amount, part.productcode);
    }
    println!("{} tree(s)", products.parts_tree(composition).await?.len());
    println!(
        "{} composition(s) use the part",
        products.compositions_all(part).await?.len()
    );

    let warehouse = client.warehouses().list_all().await?[0].idwarehouse;
    let one = NonZeroU32::new(1).unwrap();
    let production = Production::new(warehouse, one, StockReason::new("picqer-rs example")?);
    products.produce(composition, &production).await?;

    products
        .remove_part(composition, added.idproduct_part)
        .await?;

    Ok(())
}
