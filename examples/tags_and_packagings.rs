//! Makes, changes and deletes a tag, and makes and deactivates a packaging.
//!
//! Writes to the account. Picqer cannot delete a packaging, so the example deactivates it and
//! reuses it by its name.
//!
//! ```sh
//! cargo run --example tags_and_packagings
//! ```

mod common;

use picqer_rs::packagings::{NewPackaging, PackagingChanges, PackagingName, PackagingStatus};
use picqer_rs::tags::{HexColor, NewTag, TagChanges, TagTitle};

#[tokio::main]
async fn main() -> picqer_rs::Result<()> {
    let client = common::client();

    let tag = NewTag::new(
        TagTitle::new("picqer-rs example")?,
        HexColor::new("#336699")?,
        false,
    );
    let tag = client.tags().create(&tag).await?;
    println!("made tag {} ({})", tag.idtag, tag.title);
    client
        .tags()
        .update(
            tag.idtag,
            &TagChanges::new().color(HexColor::new("#cc3333")?),
        )
        .await?;
    client.tags().delete(tag.idtag).await?;

    let name = PackagingName::new("picqer-rs example")?;
    let inactive = client
        .packagings()
        .list_all(PackagingStatus::Inactive)
        .await?;
    let packaging = match inactive
        .into_iter()
        .find(|packaging| packaging.name == "picqer-rs example")
    {
        Some(existing) => existing,
        None => {
            client
                .packagings()
                .create(&NewPackaging::new(name).dimensions(30, 20, 10))
                .await?
        }
    };
    println!("packaging {}", packaging.idpackaging);
    client
        .packagings()
        .update(
            packaging.idpackaging,
            &PackagingChanges::new().active(false),
        )
        .await?;

    Ok(())
}
