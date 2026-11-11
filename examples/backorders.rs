//! Lists the backorders and makes picklists of the ones that have stock available. Reads, and
//! writes when you pass `process`.
//!
//! ```sh
//! cargo run --example backorders
//! cargo run --example backorders -- process
//! ```

mod common;

use picqer_rs::backorders::BackorderQuery;
use picqer_rs::pagination::Page;

#[tokio::main]
async fn main() -> picqer_rs::Result<()> {
    let client = common::client();
    let backorders = client.backorders();

    for backorder in backorders
        .list(&BackorderQuery::new(), Page::FIRST)
        .await?
        .iter()
        .take(5)
    {
        println!("{backorder:?}");
    }
    println!(
        "{} backorder(s)",
        backorders.list_all(&BackorderQuery::new()).await?.len()
    );

    if std::env::args().nth(1).as_deref() == Some("process") {
        backorders.process().await?;
    }

    Ok(())
}
