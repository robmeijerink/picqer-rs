//! Makes a customer with an address and a tag, changes it and deletes it.
//!
//! Writes to the account. Picqer can delete customers, so nothing is left behind.
//!
//! ```sh
//! cargo run --example customers
//! ```

mod common;

use picqer_rs::country::CountryCode;
use picqer_rs::customers::{
    AddressName, CustomerChanges, CustomerName, NewCustomer, NewCustomerAddress,
};
use picqer_rs::tags::TagQuery;

#[tokio::main]
async fn main() -> picqer_rs::Result<()> {
    let client = common::client();
    let customers = client.customers();

    let address = NewCustomerAddress::new(AddressName::new("Warehouse")?, CountryCode::new("NL")?);
    let new = NewCustomer::new(CustomerName::new("picqer-rs example")?).address(address);
    let customer = customers.create(&new).await?;
    println!("made customer {}", customer.idcustomer);

    if let Some(tag) = client.tags().list_all(&TagQuery::new()).await?.first() {
        customers.add_tag(customer.idcustomer, tag.idtag).await?;
    }

    let changed = customers
        .update(
            customer.idcustomer,
            &CustomerChanges::new().name(CustomerName::new("picqer-rs example, renamed")?),
        )
        .await?;
    println!("{} has {} tag(s)", changed.name, changed.tags.len());

    for address in customers.addresses_all(customer.idcustomer).await? {
        println!("{} {:?}", address.idcustomer_address, address.name);
    }

    customers.delete(customer.idcustomer).await?;

    Ok(())
}
