//! Reads what an account is set up with: warehouses, VAT groups, tags, pricelists, users,
//! templates, packagings, shipping providers, location types, stats and webshop orders.
//!
//! Reads only.
//!
//! ```sh
//! cargo run --example reference_data
//! ```

mod common;

use picqer_rs::packagings::PackagingStatus;
use picqer_rs::tags::TagQuery;
use picqer_rs::users::UserQuery;
use picqer_rs::webshop_orders::WebshopOrderQuery;

#[tokio::main]
async fn main() -> picqer_rs::Result<()> {
    let client = common::client();

    for warehouse in client.warehouses().list_all().await? {
        println!("warehouse {} {:?}", warehouse.idwarehouse, warehouse.name);
    }
    println!(
        "{} VAT group(s)",
        client.vat_groups().list_all().await?.len()
    );
    println!(
        "{} tag(s)",
        client.tags().list_all(&TagQuery::new()).await?.len()
    );
    println!(
        "{} pricelist(s)",
        client.pricelists().list_all().await?.len()
    );
    println!("{} template(s)", client.templates().list_all().await?.len());
    println!(
        "{} user(s), you are {:?}",
        client.users().list_all(UserQuery::new()).await?.len(),
        client.users().me().await?
    );
    println!(
        "{} packaging(s)",
        client
            .packagings()
            .list_all(PackagingStatus::Active)
            .await?
            .len()
    );
    println!(
        "{} shipping provider(s)",
        client.shipping_providers().list_all().await?.len()
    );
    println!(
        "{} location type(s)",
        client.location_types().list_all().await?.len()
    );
    println!(
        "{} packing station(s)",
        client.packing_stations().list_all().await?.len()
    );
    println!(
        "{} order field(s)",
        client.order_fields().list_all().await?.len()
    );

    for (key, description) in client.stats().list().await? {
        println!("{key:?}: {description:?}");
    }
    println!(
        "{} webshop order(s)",
        client
            .webshop_orders()
            .list_all(&WebshopOrderQuery::new())
            .await?
            .len()
    );

    Ok(())
}
