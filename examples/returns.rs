//! Registers a return for a product and follows it in its logbook.
//!
//! Writes to the account. Picqer cannot take back a return that has received products, so the
//! example deletes the return before anything is received. Pass the id of a product.
//!
//! ```sh
//! cargo run --example returns -- <product id>
//! ```

mod common;

use std::num::NonZeroU32;

use picqer_rs::returns::{NewReturn, NewReturnLog, NewReturnedProduct};
use picqer_rs::types::ProductId;

#[tokio::main]
async fn main() -> picqer_rs::Result<()> {
    let id: u64 = std::env::args()
        .nth(1)
        .and_then(|id| id.parse().ok())
        .expect("a product id");
    let client = common::client();

    let statuses = client.return_statuses().list().await?;
    let reasons = client.return_reasons().list().await?;
    let status = statuses.first().expect("the account has a return status");
    let reason = reasons.first().expect("the account has a return reason");
    println!("{} status(es), {} reason(s)", statuses.len(), reasons.len());

    let line = NewReturnedProduct::new(
        reason.idreturn_reason,
        ProductId(id),
        NonZeroU32::new(1).unwrap(),
    );
    let new = NewReturn::new(status.idreturn_status).returned_product(line);
    let made = client.returns().create(&new).await?;
    println!("return {:?} is made", made.returnid);

    client
        .returns()
        .add_log(
            made.idreturn,
            &NewReturnLog::comment("Checked by the picqer-rs example"),
        )
        .await?;
    for entry in client.returns().logs(made.idreturn).await? {
        println!("{entry:?}");
    }

    client.returns().delete(made.idreturn).await?;

    Ok(())
}
