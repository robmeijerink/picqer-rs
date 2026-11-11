//! Subscribes to an event and checks and reads what Picqer sends to your endpoint.
//!
//! Writes to the account: Picqer cannot delete a hook, so the example deactivates the one it
//! makes. Pass the address of your endpoint.
//!
//! ```sh
//! cargo run --example webhooks -- https://example.com/picqer
//! ```

mod common;

use picqer_rs::orders::Order;
use picqer_rs::webhooks::{
    HookName, NewHook, WebhookEvent, WebhookPayload, WebhookSecret, verify_signature,
};

#[tokio::main]
async fn main() -> picqer_rs::Result<()> {
    let address = std::env::args()
        .nth(1)
        .expect("the address of your endpoint");
    let client = common::client();

    let secret = WebhookSecret::new("a long secret that only you and Picqer know")?;
    let hook = NewHook::new(
        HookName::new("picqer-rs example")?,
        WebhookEvent::OrdersCreated,
        address.parse().expect("a valid url"),
    )?
    .secret(secret);
    let hook = client.hooks().create(&hook).await?;
    println!("hook {} is {:?}", hook.idhook, hook.active);

    client.hooks().deactivate(hook.idhook).await?;
    client.hooks().reactivate(hook.idhook).await?;
    client.hooks().deactivate(hook.idhook).await?;

    Ok(())
}

/// What your endpoint does with a request: check the signature on the raw body first, then read it.
#[allow(dead_code)]
fn handle(secret: &WebhookSecret, body: &[u8], signature_header: &str) {
    if verify_signature(secret, body, signature_header).is_err() {
        return;
    }
    let Ok(payload) = WebhookPayload::from_slice(body) else {
        return;
    };
    if let Ok(order) = payload.data_as::<Order>() {
        println!("{} was created", order.orderid);
    }
}
