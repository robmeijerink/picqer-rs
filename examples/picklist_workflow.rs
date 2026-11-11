//! Takes a new picklist through the warehouse: pick, ship and close.
//!
//! Writes to the account. It uses the first new picklist and the first shipping profile that
//! Picqer allows for it, so use a development account. A provider that needs a real contract
//! answers with `ErrorCode::CannotCreateShipment`.
//!
//! ```sh
//! cargo run --example picklist_workflow
//! ```

mod common;

use std::num::NonZeroU32;

use picqer_rs::pagination::Page;
use picqer_rs::picklists::{Pick, PickTarget, PicklistQuery, PicklistStatus};
use picqer_rs::shipments::NewShipment;
use picqer_rs::types::{Grams, ProductCode};

#[tokio::main]
async fn main() -> picqer_rs::Result<()> {
    let client = common::client();
    let picklists = client.picklists();

    let query = PicklistQuery::new().status(PicklistStatus::New);
    let Some(found) = picklists
        .list(&query, Page::FIRST)
        .await?
        .into_iter()
        .next()
    else {
        println!("there is no new picklist");
        return Ok(());
    };
    let picklist = picklists.get(found.idpicklist).await?;
    println!(
        "{} has {} line(s)",
        picklist.picklistid,
        picklist.products.len()
    );

    let line = picklist.products.first().expect("the picklist has a line");
    let one = NonZeroU32::new(1).unwrap();
    let target = PickTarget::Product(ProductCode::new(&line.productcode)?);
    let picked = picklists
        .pick(picklist.idpicklist, &Pick::new(target.clone(), one))
        .await?;
    println!("{} picked", picked.totalpicked);
    picklists
        .pick(picklist.idpicklist, &Pick::undo(target, one))
        .await?;

    picklists.pick_all(picklist.idpicklist).await?;

    let methods = client.shipments().methods(picklist.idpicklist).await?;
    let method = methods
        .first()
        .expect("a shipping method is allowed for the picklist");
    let shipment = NewShipment::new(method.idshippingprovider_profile).weight(Grams::new(900));
    let shipped = client
        .shipments()
        .create(picklist.idpicklist, &shipment)
        .await?;
    println!(
        "shipment {} has tracking code {:?}",
        shipped.idshipment, shipped.parcels
    );

    let closed = picklists.close(picklist.idpicklist).await?;
    println!("{} is {:?}", closed.picklistid, closed.status);

    Ok(())
}
