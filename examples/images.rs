//! Adds an image to a product from a file and removes it again.
//!
//! Writes to the account. The format (JPEG, PNG or WebP) is recognised from the bytes.
//!
//! ```sh
//! cargo run --example images -- <product id> photo.jpg
//! ```

mod common;

use picqer_rs::products::NewProductImage;
use picqer_rs::types::ProductId;

#[tokio::main]
async fn main() -> picqer_rs::Result<()> {
    let mut args = std::env::args().skip(1);
    let product = ProductId(
        args.next()
            .and_then(|id| id.parse().ok())
            .expect("a product id"),
    );
    let path = args.next().expect("the path of an image");
    let bytes = std::fs::read(&path).expect("the image can be read");

    let client = common::client();
    let products = client.products();

    let added = products
        .add_image(product, &NewProductImage::new(bytes)?)
        .await?;
    println!("added {added:?}");
    println!("{} image(s)", products.images_all(product).await?.len());
    products
        .remove_image(product, added.idproduct_image)
        .await?;

    Ok(())
}
