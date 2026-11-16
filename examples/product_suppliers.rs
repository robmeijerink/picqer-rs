//! Links a supplier to a product, changes its purchase details and removes it again.
//!
//! Writes to the account. A product can have more than one supplier; an account of Picqer
//! Fulfilment can link one per product and refuses a second one.
//!
//! ```sh
//! cargo run --example product_suppliers -- <product id> <supplier id>
//! ```

mod common;

use picqer_rs::products::{NewProductSupplier, ProductSupplierChanges, SupplierProductCode};
use picqer_rs::types::{ProductId, SupplierId};

#[tokio::main]
async fn main() -> picqer_rs::Result<()> {
    let mut args = std::env::args().skip(1);
    let product = ProductId(
        args.next()
            .and_then(|id| id.parse().ok())
            .expect("a product id"),
    );
    let supplier = SupplierId(
        args.next()
            .and_then(|id| id.parse().ok())
            .expect("a supplier id"),
    );

    let client = common::client();
    let products = client.products();

    let linked = products
        .add_supplier(
            product,
            &NewProductSupplier::new(supplier)
                .productcode(SupplierProductCode::new("SUP-6531-RB")?)
                .deliverytime(5),
        )
        .await?;
    println!("linked {linked:?}");

    let changed = products
        .update_supplier(
            product,
            linked.idproduct_supplier,
            &ProductSupplierChanges::new()
                .deliverytime(3)
                .without_productcode(),
        )
        .await?;
    println!("changed {changed:?}");

    for entry in products.suppliers(product).await? {
        println!("{:?} default: {:?}", entry.idsupplier, entry.is_default);
    }

    products
        .remove_supplier(product, linked.idproduct_supplier)
        .await?;

    Ok(())
}
