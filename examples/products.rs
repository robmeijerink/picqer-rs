//! Makes a product, finds it again, changes it and makes it inactive.
//!
//! Writes to the account. Picqer cannot delete a product, so the example reuses the product
//! with the code `picqer-rs-example` when it exists.
//!
//! ```sh
//! cargo run --example products
//! ```

mod common;

use picqer_rs::pagination::Page;
use picqer_rs::products::{NewProduct, ProductChanges, ProductQuery, ProductStatus};
use picqer_rs::types::{Money, ProductCode, ProductName};

#[tokio::main]
async fn main() -> picqer_rs::Result<()> {
    let client = common::client();
    let products = client.products();

    for product in products
        .list(&ProductQuery::new(), Page::FIRST)
        .await?
        .iter()
        .take(3)
    {
        println!(
            "{} {} {}",
            product.idproduct, product.productcode, product.name
        );
    }

    let code = ProductCode::new("picqer-rs-example")?;
    let product = match products.find_by_productcode(&code).await? {
        Some(existing) => existing,
        None => {
            let vat_group = client.vat_groups().list_all().await?[0].idvatgroup;
            let new = NewProduct::new(
                vat_group,
                code.clone(),
                ProductName::new("picqer-rs example")?,
                Money::from_cents(1250)?,
            );
            products.create(&new).await?
        }
    };
    println!("working with product {}", product.idproduct);

    let changes = ProductChanges::new()
        .name(ProductName::new("picqer-rs example, renamed")?)
        .price(Money::from_cents(1500)?);
    let changed = products.update(product.idproduct, &changes).await?;
    println!("{} costs {:?}", changed.name, changed.price);

    products.inactivate(product.idproduct).await?;
    let inactive = products
        .list(
            &ProductQuery::new()
                .status(ProductStatus::Inactive)
                .productcode(code),
            Page::FIRST,
        )
        .await?;
    println!("{} inactive product(s) have that code", inactive.len());

    Ok(())
}
