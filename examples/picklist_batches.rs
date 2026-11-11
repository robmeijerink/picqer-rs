//! Lists the open picklist batches and moves a picklist into one and out again.
//!
//! Writes to the account, but leaves the batch as it was. Picqer cannot delete a batch and
//! completes one as soon as it has no picklists left, so the example does not make batches.
//!
//! ```sh
//! cargo run --example picklist_batches
//! ```

mod common;

use picqer_rs::pagination::Page;
use picqer_rs::picklist_batch_presets::PresetStatus;
use picqer_rs::picklist_batches::{BatchQuery, BatchStatus};
use picqer_rs::picklists::{PicklistQuery, PicklistStatus};

#[tokio::main]
async fn main() -> picqer_rs::Result<()> {
    let client = common::client();
    let batches = client.picklist_batches();

    let open = batches
        .list(&BatchQuery::new().status(BatchStatus::Open), Page::FIRST)
        .await?;
    for batch in &open {
        println!("{} {:?}", batch.idpicklist_batch, batch.picklist_batchid);
    }
    let Some(batch) = open.first() else {
        println!("there is no open batch");
        return Ok(());
    };

    let query = PicklistQuery::new().status(PicklistStatus::New);
    let Some(picklist) = client
        .picklists()
        .list(&query, Page::FIRST)
        .await?
        .into_iter()
        .find(|picklist| picklist.idpicklist_batch.is_none())
    else {
        println!("there is no new picklist outside a batch");
        return Ok(());
    };

    let with = batches
        .add_picklist(batch.idpicklist_batch, picklist.idpicklist)
        .await?;
    println!(
        "batch {} now has {} picklist(s)",
        with.idpicklist_batch,
        with.picklists.len()
    );

    let without = batches
        .remove_picklist(batch.idpicklist_batch, picklist.idpicklist)
        .await?;
    println!("and {} after taking it out", without.picklists.len());

    let presets = client
        .picklist_batch_presets()
        .list_all(PresetStatus::Active)
        .await?;
    println!("{} preset(s)", presets.len());

    Ok(())
}
