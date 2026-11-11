//! Makes a picking container, links a picklist to it and deletes it again.
//!
//! Writes to the account. A container holds one picklist: linking another one takes the place of
//! the first.
//!
//! ```sh
//! cargo run --example picking_containers
//! ```

mod common;

use picqer_rs::pagination::Page;
use picqer_rs::picking_containers::{PickingContainerName, PickingContainerQuery};
use picqer_rs::picklists::{PicklistQuery, PicklistStatus};

#[tokio::main]
async fn main() -> picqer_rs::Result<()> {
    let client = common::client();
    let containers = client.picking_containers();

    let made = containers
        .create(&PickingContainerName::new("picqer-rs example")?)
        .await?;
    println!("made {:?}", made);

    let renamed = containers
        .rename(
            made.idpicking_container,
            &PickingContainerName::new("picqer-rs example 2")?,
        )
        .await?;
    println!("renamed to {:?}", renamed.name);

    let query = PicklistQuery::new().status(PicklistStatus::New);
    if let Some(picklist) = client.picklists().list(&query, Page::FIRST).await?.first() {
        containers
            .link(made.idpicking_container, picklist.idpicklist)
            .await?;
        let linked = containers
            .list(
                PickingContainerQuery::new().idpicklist(picklist.idpicklist),
                Page::FIRST,
            )
            .await?;
        println!("{} container(s) hold the picklist", linked.len());
    }

    containers.delete(made.idpicking_container).await?;

    Ok(())
}
