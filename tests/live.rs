//! Runs against a real Picqer account.
//!
//! ```sh
//! PICQER_SUBDOMAIN=... PICQER_API_KEY=... cargo test --test live -- --ignored
//! ```
//!
//! Use an account meant for development. The product test reuses one inactive product with the
//! code `picqer-rs-live`, because Picqer cannot delete products.

use std::time::{SystemTime, UNIX_EPOCH};

use picqer_rs::comments::{
    CommentBody, CommentQuery, CommentSource, CommentTarget, Commenter, NewComment, Person,
};
use picqer_rs::country::CountryCode;
use picqer_rs::fields::FieldType;
use picqer_rs::pricelists::PricelistPrice;
use std::num::NonZeroI64;

use picqer_rs::products::{
    ImageType, NewProduct, NewProductImage, ProductChanges, ProductQuery, ProductStatus,
    ProductType, WarehouseSettingsChanges,
};
use picqer_rs::stock::{StockReason, StockTarget, StockUpdate};
use picqer_rs::stock_history::{ChangeType, StockHistoryQuery};
use picqer_rs::tags::{HexColor, NewTag, TagChanges, TagQuery, TagTitle};
use picqer_rs::types::HookId;
use picqer_rs::types::{Grams, Money, ProductCode, ProductName};
use picqer_rs::{Client, Reported, UserAgent};

const PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78, 0xDA, 0x63, 0x64, 0x60, 0xF8, 0x5F,
    0x0F, 0x00, 0x02, 0x87, 0x01, 0x80, 0xEB, 0x47, 0xBA, 0x92, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45,
    0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

/// Loads a `.env` file from the working directory, once, before the first variable is read.
/// Variables that are already set win over the file.
fn load_env() {
    static LOAD: std::sync::Once = std::sync::Once::new();
    LOAD.call_once(|| {
        dotenvy::dotenv().ok();
    });
}

fn client() -> Client {
    load_env();
    let user_agent =
        UserAgent::new("picqer-rs live tests", "github.com/robmeijerink/picqer-rs").unwrap();
    Client::builder(
        picqer_rs::Subdomain::from_env().expect("PICQER_SUBDOMAIN is set"),
        picqer_rs::ApiKey::from_env().expect("PICQER_API_KEY is set"),
        user_agent,
    )
    .retry_on_rate_limit(std::num::NonZeroU32::new(3).unwrap())
    .throttle()
    .build()
    .unwrap()
}

/// The tests that make locations or change the default location type, which would change what the
/// others see when they run at the same time.
static LOCATION_TESTS: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn unique(prefix: &str) -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("{prefix}-{seconds}")
}

#[tokio::test]
#[ignore = "needs a Picqer account"]
async fn every_reading_endpoint_returns_what_the_types_expect() {
    let client = client();

    let warehouses = client.warehouses().list_all().await.unwrap();
    assert!(!warehouses.is_empty());
    let warehouse = &warehouses[0];
    assert_eq!(
        &client
            .warehouses()
            .get(warehouse.idwarehouse)
            .await
            .unwrap(),
        warehouse
    );
    client
        .warehouses()
        .stock_all(warehouse.idwarehouse)
        .await
        .unwrap();

    let vat_groups = client.vat_groups().list_all().await.unwrap();
    assert!(!vat_groups.is_empty());
    assert_eq!(
        client
            .vat_groups()
            .get(vat_groups[0].idvatgroup)
            .await
            .unwrap(),
        vat_groups[0]
    );

    let tags = client.tags().list_all(&TagQuery::new()).await.unwrap();
    assert_eq!(client.tags().get(tags[0].idtag).await.unwrap(), tags[0]);

    let customers = client
        .customers()
        .list_all(&picqer_rs::customers::CustomerQuery::new())
        .await
        .unwrap();
    assert!(!customers.is_empty());
    let customer = &customers[0];
    assert!(customer.customerfields.is_empty());
    let mut single = client.customers().get(customer.idcustomer).await.unwrap();
    single.customerfields.clear();
    assert_eq!(&single, customer);
    client
        .customers()
        .addresses_all(customer.idcustomer)
        .await
        .unwrap();
    client.customers().tags(customer.idcustomer).await.unwrap();
    client.customer_fields().list_all().await.unwrap();
    for field in client.order_fields().list_all().await.unwrap() {
        client.order_fields().get(field.idorderfield).await.unwrap();
    }

    let only_reader_of_locations = LOCATION_TESTS.lock().await;
    client
        .webshop_orders()
        .list_all(&picqer_rs::webshop_orders::WebshopOrderQuery::new())
        .await
        .unwrap();

    let stats = client.stats().list().await.unwrap();
    assert_eq!(stats.len(), picqer_rs::stats::StatKey::ALL.len());
    for (key, description) in &stats {
        let Reported::Known(key) = key else {
            panic!("the key {key:?} is not known to the crate");
        };
        assert!(!description.description.is_empty());
        assert!(client.stats().get(*key).await.unwrap() >= 0);
    }
    assert!(
        client
            .stats()
            .get_by_key("bogus")
            .await
            .unwrap_err()
            .is_not_found()
    );

    for location_type in client.location_types().list_all().await.unwrap() {
        assert_eq!(
            client
                .location_types()
                .get(location_type.idlocation_type)
                .await
                .unwrap(),
            location_type
        );
    }
    let locations = client
        .locations()
        .list(
            &picqer_rs::locations::LocationQuery::new(),
            picqer_rs::pagination::Page::FIRST,
        )
        .await
        .unwrap();
    assert!(!locations.is_empty());
    assert_eq!(
        &client
            .locations()
            .get(locations[0].idlocation)
            .await
            .unwrap(),
        &locations[0]
    );
    for container in client
        .locations()
        .list_all(
            &picqer_rs::locations::LocationQuery::new()
                .kind(picqer_rs::locations::LocationKind::Container),
        )
        .await
        .unwrap()
    {
        assert_eq!(
            client.locations().get(container.idlocation).await.unwrap(),
            container
        );
        client
            .locations()
            .products_all(container.idlocation)
            .await
            .unwrap();
    }
    for location in locations.iter().take(20) {
        client
            .locations()
            .products_all(location.idlocation)
            .await
            .unwrap();
    }
    let history = client
        .location_stock_history()
        .list_all(&picqer_rs::location_stock_history::LocationStockHistoryQuery::new())
        .await
        .unwrap();
    for entry in history.iter().take(3) {
        assert_eq!(
            &client
                .location_stock_history()
                .get(entry.idproduct_location_stock_history)
                .await
                .unwrap(),
            entry
        );
    }
    drop(only_reader_of_locations);

    let returns = client
        .returns()
        .list_all(&picqer_rs::returns::ReturnQuery::new())
        .await
        .unwrap();
    assert!(!returns.is_empty());
    for one in returns.iter().take(5) {
        let full = client.returns().get(one.idreturn).await.unwrap();
        assert_eq!(full.idreturn, one.idreturn);
        assert_eq!(full.returnid, one.returnid);
        assert_eq!(
            client
                .returns()
                .returned_products(one.idreturn)
                .await
                .unwrap(),
            full.returned_products
        );
        assert_eq!(
            client
                .returns()
                .replacement_products(one.idreturn)
                .await
                .unwrap(),
            full.replacement_products
        );
        assert!(
            !client
                .returns()
                .logs(one.idreturn)
                .await
                .unwrap()
                .is_empty()
        );
    }

    let orders = client
        .orders()
        .list_all(&picqer_rs::orders::OrderQuery::new())
        .await
        .unwrap();
    assert!(!orders.is_empty());
    let order = &orders[0];
    assert_eq!(
        client.orders().get(order.idorder).await.unwrap().orderid,
        order.orderid
    );
    let number = picqer_rs::types::OrderNumber::new(order.orderid.clone()).unwrap();
    let found = client.orders().find_by_number(&number).await.unwrap();
    assert_eq!(found.map(|found| found.idorder), Some(order.idorder));

    for status in [
        picqer_rs::picklists::PicklistStatus::New,
        picqer_rs::picklists::PicklistStatus::Cancelled,
    ] {
        let picklists = client
            .picklists()
            .list_all(&picqer_rs::picklists::PicklistQuery::new().status(status))
            .await
            .unwrap();
        assert!(
            picklists
                .iter()
                .all(|picklist| matches!(picklist.status, Reported::Known(_)))
        );
    }
    let open = client
        .picklists()
        .list_all(
            &picqer_rs::picklists::PicklistQuery::new()
                .status(picqer_rs::picklists::PicklistStatus::New),
        )
        .await
        .unwrap();
    let picklist = open
        .iter()
        .min_by_key(|picklist| picklist.idpicklist)
        .unwrap();
    let fetched = client.picklists().get(picklist.idpicklist).await.unwrap();
    assert_eq!(fetched.picklistid, picklist.picklistid);
    assert!(!fetched.products.is_empty());

    let suppliers = client.suppliers().list_all().await.unwrap();
    assert!(!suppliers.is_empty());
    let supplier = &suppliers[0];
    assert_eq!(
        client
            .suppliers()
            .get(supplier.idsupplier)
            .await
            .unwrap()
            .name,
        supplier.name
    );

    let purchase_orders = client
        .purchase_orders()
        .list_all(&picqer_rs::purchase_orders::PurchaseOrderQuery::new())
        .await
        .unwrap();
    assert!(!purchase_orders.is_empty());
    let purchase_order = &purchase_orders[0];
    let fetched = client
        .purchase_orders()
        .get(purchase_order.idpurchaseorder)
        .await
        .unwrap();
    assert_eq!(fetched.purchaseorderid, purchase_order.purchaseorderid);
    assert_eq!(
        client
            .purchase_orders()
            .lines(purchase_order.idpurchaseorder)
            .await
            .unwrap()
            .len(),
        purchase_order.products.len()
    );

    let me = client.users().me().await.unwrap();
    assert!(!me.rights.is_empty());
    assert!(
        !client
            .users()
            .list_all(picqer_rs::users::UserQuery::new())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(!client.templates().list_all().await.unwrap().is_empty());
    client.shipping_providers().list_all().await.unwrap();
    assert!(
        !client
            .packagings()
            .list_all(picqer_rs::packagings::PackagingStatus::Active)
            .await
            .unwrap()
            .is_empty()
    );

    client.pricelists().list_all().await.unwrap();
    client.product_fields().list_all().await.unwrap();

    let products = client
        .products()
        .list_all(&ProductQuery::new())
        .await
        .unwrap();
    assert!(!products.is_empty());
    let product = &products[0];
    let fetched = client.products().get(product.idproduct).await.unwrap();
    assert_eq!(fetched.productcode, product.productcode);
    assert_eq!(fetched.price, product.price);
    let code = ProductCode::new(product.productcode.clone()).unwrap();
    let found = client.products().find_by_productcode(&code).await.unwrap();
    assert_eq!(found.map(|found| found.idproduct), Some(product.idproduct));
    client.products().tags(product.idproduct).await.unwrap();

    let error = client
        .products()
        .parts(product.idproduct)
        .await
        .unwrap_err();
    assert!(error.is_not_found());
    assert!(
        client
            .products()
            .compositions_all(product.idproduct)
            .await
            .unwrap()
            .is_empty()
    );

    client
        .products()
        .images_all(product.idproduct)
        .await
        .unwrap();
    client
        .products()
        .price_history(product.idproduct)
        .await
        .unwrap();
    client
        .products()
        .fixed_stock_price_history(product.idproduct)
        .await
        .unwrap();
    client.products().expected(product.idproduct).await.unwrap();
    let settings = client
        .products()
        .warehouse_settings(product.idproduct)
        .await
        .unwrap();
    assert!(!settings.is_empty());

    let rate_limit = client.rate_limit().expect("the API reports the rate limit");
    assert!(rate_limit.limit() > 0);
}

#[tokio::test]
#[ignore = "needs a Picqer account"]
async fn a_location_type_can_be_made_changed_and_deleted() {
    let _only_test_with_locations = LOCATION_TESTS.lock().await;
    use picqer_rs::location_types::{LocationTypeChanges, LocationTypeName, NewLocationType};
    use picqer_rs::tags::HexColor;

    let client = client();
    let types = client.location_types();
    let name = "picqer-rs-live-type";
    for old in types.list_all().await.unwrap() {
        if old.name.starts_with(name) {
            types.delete(old.idlocation_type).await.unwrap();
        }
    }

    let original_default = types
        .list_all()
        .await
        .unwrap()
        .into_iter()
        .find(|location_type| location_type.default == Some(true))
        .expect("the account has a default location type")
        .idlocation_type;
    let made = types
        .create(&NewLocationType::new(
            LocationTypeName::new(name).unwrap(),
            HexColor::new("#FF0000").unwrap(),
        ))
        .await
        .unwrap();
    let result = async {
        assert_eq!(made.name, name);
        assert_eq!(made.color, "#FF0000");
        assert_ne!(made.default, Some(true));
        let duplicate = types
            .create(&NewLocationType::new(
                LocationTypeName::new(name).unwrap(),
                HexColor::new("#00FF00").unwrap(),
            ))
            .await
            .unwrap_err();
        assert_eq!(duplicate.as_api().unwrap().status, 400);

        let changed = types
            .update(
                made.idlocation_type,
                &LocationTypeChanges::new()
                    .name(LocationTypeName::new("picqer-rs-live-type-2").unwrap())
                    .color(HexColor::new("#00FF00").unwrap()),
            )
            .await?;
        assert_eq!(changed.name, "picqer-rs-live-type-2");
        assert_eq!(changed.color, "#00FF00");
        assert_eq!(changed.default, Some(false));
        assert_eq!(types.get(made.idlocation_type).await?, changed);
        assert!(types.list_all().await?.contains(&changed));

        let default = types.set_default(made.idlocation_type).await?;
        assert_eq!(default.default, Some(true));
        let all = types.list_all().await?;
        assert_eq!(
            all.iter()
                .filter(|location_type| location_type.default == Some(true))
                .map(|location_type| location_type.idlocation_type)
                .collect::<Vec<_>>(),
            vec![made.idlocation_type]
        );
        let refused = types.delete(made.idlocation_type).await.unwrap_err();
        assert_eq!(
            refused.as_api().unwrap().code,
            Some(Reported::Known(picqer_rs::ErrorCode::MissingParameters))
        );
        Ok::<(), picqer_rs::Error>(())
    }
    .await;
    types.set_default(original_default).await.unwrap();
    types.delete(made.idlocation_type).await.unwrap();
    result.unwrap();
    assert!(
        types
            .get(made.idlocation_type)
            .await
            .unwrap_err()
            .is_not_found()
    );
}

#[tokio::test]
#[ignore = "needs a Picqer account"]
async fn a_location_and_a_container_can_be_made_changed_and_deleted() {
    let _only_test_with_locations = LOCATION_TESTS.lock().await;
    use picqer_rs::locations::{
        LocationChanges, LocationKind, LocationName, LocationQuery, NewContainer, NewLocation,
    };

    let client = client();
    let locations = client.locations();
    let warehouse = client.warehouses().list_all().await.unwrap()[0].idwarehouse;
    let location_type = client
        .location_types()
        .list_all()
        .await
        .unwrap()
        .into_iter()
        .find(|location_type| !location_type.name.starts_with("picqer-rs-live"))
        .expect("the account has a location type of its own")
        .idlocation_type;
    let parent = locations
        .list(&LocationQuery::new(), picqer_rs::pagination::Page::FIRST)
        .await
        .unwrap()[0]
        .idlocation;
    let names = ["ZZP.1.2", "ZZP.1", "ZZP", "ZZP-live", "ZZP-live-2"];
    let remove_leftovers = || async {
        let mut found = Vec::new();
        for name in names {
            found.extend(
                locations
                    .list_all(&LocationQuery::new().name(name))
                    .await
                    .unwrap(),
            );
        }
        found.sort_by_key(|location| std::cmp::Reverse(location.idlocation));
        for location in found {
            locations.delete(location.idlocation).await.unwrap();
        }
    };
    remove_leftovers().await;
    let before = locations
        .list_all(&LocationQuery::new().idwarehouse(warehouse))
        .await
        .unwrap()
        .len();

    let made = locations
        .create(
            &NewLocation::new(LocationName::new("ZZP-live").unwrap(), warehouse)
                .remarks("live test")
                .is_bulk_location(true),
        )
        .await
        .unwrap();
    let nested = locations
        .create(
            &NewLocation::new(LocationName::new("ZZP.1.2").unwrap(), warehouse)
                .auto_link_to_parent(),
        )
        .await
        .unwrap();
    let container = locations
        .create_container(NewContainer::new(warehouse))
        .await
        .unwrap();

    let result = async {
        assert_eq!(made.name, "ZZP-live");
        assert_eq!(made.kind, Some(Reported::Known(LocationKind::Location)));
        assert_eq!(made.idwarehouse, warehouse);
        assert_eq!(made.parent_idlocation, None);
        assert!(made.location_type.is_some());
        let stored = locations.get(made.idlocation).await?;
        assert_eq!(stored.remarks.as_deref(), Some("live test"));
        assert_eq!(stored.is_bulk_location, Some(true));
        assert_eq!(stored.unlink_on_empty, Some(false));

        assert_eq!(nested.name, "ZZP.1.2");
        let parent_of_nested = locations
            .get(
                nested
                    .parent_idlocation
                    .expect("auto_link_to_parent made a parent"),
            )
            .await?;
        assert_eq!(parent_of_nested.name, "ZZP.1");
        assert_eq!(
            locations
                .get(
                    parent_of_nested
                        .parent_idlocation
                        .expect("and that parent has one"),
                )
                .await?
                .name,
            "ZZP"
        );

        assert_eq!(
            container.kind,
            Some(Reported::Known(LocationKind::Container))
        );
        assert!(container.name.starts_with("Container"));
        let remarked = locations
            .update(
                container.idlocation,
                &LocationChanges::new().remarks("live container"),
            )
            .await?;
        assert_eq!(remarked.remarks.as_deref(), Some("live container"));
        assert_eq!(remarked.name, container.name);
        assert!(
            locations
                .list_all(&LocationQuery::new().kind(LocationKind::Container))
                .await?
                .iter()
                .any(|found| found.idlocation == container.idlocation)
        );
        assert!(
            locations
                .products_all(container.idlocation)
                .await?
                .is_empty()
        );

        let changed = locations
            .update(
                made.idlocation,
                &LocationChanges::new()
                    .name(LocationName::new("ZZP-live-2").unwrap())
                    .without_remarks()
                    .parent_idlocation(parent)
                    .idlocation_type(location_type)
                    .unlink_on_empty(true)
                    .is_bulk_location(false)
                    .is_exclusive_location(true),
            )
            .await?;
        assert_eq!(changed.name, "ZZP-live-2");
        assert_eq!(changed.remarks, None);
        assert_eq!(changed.parent_idlocation, Some(parent));
        assert_eq!(changed.unlink_on_empty, Some(true));
        assert_eq!(changed.is_bulk_location, Some(false));
        assert_eq!(changed.is_exclusive_location, Some(true));
        assert!(locations.products_all(made.idlocation).await?.is_empty());
        let loose = locations
            .update(made.idlocation, &LocationChanges::new().without_parent())
            .await?;
        assert_eq!(loose.parent_idlocation, None);

        let found = locations
            .list_all(&LocationQuery::new().name("ZZP-live-2"))
            .await?;
        assert_eq!(found.len(), 1);
        assert!(
            locations
                .list_all(&LocationQuery::new().name("ZZP-live"))
                .await?
                .is_empty()
        );
        let duplicate = locations
            .create(&NewLocation::new(
                LocationName::new("ZZP-live-2").unwrap(),
                warehouse,
            ))
            .await
            .unwrap_err();
        assert_eq!(duplicate.as_api().unwrap().status, 400);
        Ok::<(), picqer_rs::Error>(())
    }
    .await;

    locations.delete(container.idlocation).await.unwrap();
    remove_leftovers().await;
    result.unwrap();
    for name in names {
        assert!(
            locations
                .list_all(&LocationQuery::new().name(name))
                .await
                .unwrap()
                .is_empty(),
            "{name} was left behind"
        );
    }
    assert_eq!(
        locations
            .list_all(&LocationQuery::new().idwarehouse(warehouse))
            .await
            .unwrap()
            .len(),
        before
    );
}

#[tokio::test]
#[ignore = "needs a Picqer account"]
async fn a_product_can_be_linked_to_locations_and_its_stock_moved() {
    let _only_test_with_locations = LOCATION_TESTS.lock().await;
    use picqer_rs::locations::{
        LocationChanges, LocationName, LocationQuery, NewContainer, NewLocation,
    };
    use picqer_rs::products::{LotNumber, NewProductLocation, ProductLocationChanges};
    use picqer_rs::stock::{StockMove, StockPlace};
    use std::num::{NonZeroI64, NonZeroU32};

    let client = client();
    let products = client.products();
    let locations = client.locations();
    let all_warehouses = client.warehouses().list_all().await.unwrap();
    let warehouse = all_warehouses[0].idwarehouse;
    let other_warehouse = all_warehouses
        .iter()
        .map(|warehouse| warehouse.idwarehouse)
        .find(|id| *id != warehouse)
        .expect("the account has a second warehouse");
    let reason = || StockReason::new("picqer-rs live test").unwrap();
    let code = ProductCode::new("picqer-rs-live-locations").unwrap();
    let product = match products.find_by_productcode(&code).await.unwrap() {
        Some(existing) => {
            products.activate(existing.idproduct).await.unwrap();
            existing.idproduct
        }
        None => {
            let vat_group = client.vat_groups().list_all().await.unwrap()[0].idvatgroup;
            products
                .create(&NewProduct::new(
                    vat_group,
                    code.clone(),
                    ProductName::new("picqer-rs live test locations").unwrap(),
                    Money::from_cents(100).unwrap(),
                ))
                .await
                .unwrap()
                .idproduct
        }
    };
    let names = ["ZZP-pl1", "ZZP-pl2", "ZZP-pl3"];
    let clean_up = || async {
        products
            .update(
                product,
                &ProductChanges::new()
                    .track_lot_numbers(false)
                    .track_expiration_dates(false),
            )
            .await
            .unwrap();
        let mut linked = products.locations(product).await.unwrap();
        for entry in &linked {
            let amount = entry
                .stock_for_product
                .as_ref()
                .and_then(|stock| stock.stock)
                .unwrap_or(0);
            if let Ok(amount) = u32::try_from(amount)
                && let Some(amount) = NonZeroU32::new(amount)
            {
                products
                    .move_stock(
                        product,
                        &StockMove::new(
                            amount,
                            StockPlace::Location(entry.location.idlocation),
                            StockPlace::NoSpecificLocation(entry.location.idwarehouse),
                        ),
                    )
                    .await
                    .unwrap();
            }
        }
        linked = products.locations(product).await.unwrap();
        for entry in linked {
            products
                .unlink_location(product, entry.location.idlocation)
                .await
                .unwrap();
        }
        for name in names {
            for old in locations
                .list_all(&LocationQuery::new().name(name))
                .await
                .unwrap()
            {
                locations.delete(old.idlocation).await.unwrap();
            }
        }
        for container in locations
            .list_all(&LocationQuery::new().kind(picqer_rs::locations::LocationKind::Container))
            .await
            .unwrap()
        {
            if container.remarks.as_deref() == Some("picqer-rs-live") {
                locations.delete(container.idlocation).await.unwrap();
            }
        }
        for id in [warehouse, other_warehouse] {
            products
                .update_stock(product, id, &StockUpdate::set(0, reason()))
                .await
                .unwrap();
        }
    };
    clean_up().await;

    let first = locations
        .create(&NewLocation::new(
            LocationName::new(names[0]).unwrap(),
            warehouse,
        ))
        .await
        .unwrap();
    let second = locations
        .create(&NewLocation::new(
            LocationName::new(names[1]).unwrap(),
            warehouse,
        ))
        .await
        .unwrap();
    let third = locations
        .create(&NewLocation::new(
            LocationName::new(names[2]).unwrap(),
            other_warehouse,
        ))
        .await
        .unwrap();
    let container = locations
        .create_container(NewContainer::new(warehouse))
        .await
        .unwrap();
    locations
        .update(
            container.idlocation,
            &LocationChanges::new().remarks("picqer-rs-live"),
        )
        .await
        .unwrap();

    let result = async {
        assert!(products.locations(product).await?.is_empty());
        let linked = products
            .link_location(
                product,
                &NewProductLocation::new(first.idlocation)
                    .note("top shelf")
                    .preferred(),
            )
            .await?;
        assert_eq!(linked.location.idlocation, first.idlocation);
        assert_eq!(linked.note.as_deref(), Some("top shelf"));
        assert_eq!(linked.is_preferred, Some(true));
        let again = products
            .link_location(product, &NewProductLocation::new(first.idlocation))
            .await?;
        assert_eq!(again.location.idlocation, first.idlocation);
        products
            .link_location(product, &NewProductLocation::new(second.idlocation))
            .await?;
        let both = products.locations(product).await?;
        assert_eq!(both.len(), 2);

        let moved_preference = products
            .update_location(
                product,
                second.idlocation,
                &ProductLocationChanges::new().preferred().note("second"),
            )
            .await?;
        assert_eq!(moved_preference.is_preferred, Some(true));
        let after = products.locations(product).await?;
        let preferred: Vec<_> = after
            .iter()
            .filter(|entry| entry.is_preferred == Some(true))
            .map(|entry| entry.location.idlocation)
            .collect();
        assert_eq!(preferred, vec![second.idlocation]);
        let cleared = products
            .update_location(
                product,
                first.idlocation,
                &ProductLocationChanges::new().without_note(),
            )
            .await?;
        assert_eq!(cleared.note, None);

        products
            .update_stock(
                product,
                warehouse,
                &StockUpdate::adjust(
                    StockTarget::NoSpecificLocation,
                    NonZeroI64::new(10).unwrap(),
                    reason(),
                ),
            )
            .await?;
        let two = NonZeroU32::new(2).unwrap();
        let one = NonZeroU32::new(1).unwrap();
        let moved = products
            .move_stock(
                product,
                &StockMove::new(
                    two,
                    StockPlace::NoSpecificLocation(warehouse),
                    StockPlace::Location(first.idlocation),
                ),
            )
            .await?;
        assert_eq!(moved.amount, 2);
        assert!(moved.from.location.is_none());
        assert_eq!(
            moved.to.location.as_ref().unwrap().idlocation,
            first.idlocation
        );
        assert_eq!(moved.to.warehouse.level.stock, 10);
        let at_first = |detail: &picqer_rs::stock::WarehouseStockDetail, id| {
            detail
                .locations
                .iter()
                .find(|entry| entry.idlocation == id)
                .map(|entry| entry.stock)
        };
        assert_eq!(at_first(&moved.to.warehouse, first.idlocation), Some(2));

        let between = products
            .move_stock(
                product,
                &StockMove::new(
                    one,
                    StockPlace::Location(first.idlocation),
                    StockPlace::Location(second.idlocation),
                )
                .reason(reason()),
            )
            .await?;
        assert_eq!(at_first(&between.to.warehouse, first.idlocation), Some(1));
        assert_eq!(at_first(&between.to.warehouse, second.idlocation), Some(1));
        let stock_of = |entries: &[picqer_rs::products::ProductLocation], id| {
            entries
                .iter()
                .find(|entry| entry.location.idlocation == id)
                .and_then(|entry| entry.stock_for_product.as_ref())
                .and_then(|stock| stock.stock)
        };
        let now = products.locations(product).await?;
        assert_eq!(stock_of(&now, first.idlocation), Some(1));
        assert_eq!(stock_of(&now, second.idlocation), Some(1));

        let too_much = products
            .move_stock(
                product,
                &StockMove::new(
                    NonZeroU32::new(99).unwrap(),
                    StockPlace::Location(first.idlocation),
                    StockPlace::Location(second.idlocation),
                ),
            )
            .await
            .unwrap_err();
        assert_eq!(
            too_much.as_api().unwrap().code,
            Some(Reported::Known(picqer_rs::ErrorCode::NegativeStock))
        );
        let same = products
            .move_stock(
                product,
                &StockMove::new(
                    one,
                    StockPlace::Location(first.idlocation),
                    StockPlace::Location(first.idlocation),
                ),
            )
            .await
            .unwrap_err();
        assert_eq!(same.as_api().unwrap().status, 400);

        let refused = products
            .unlink_location(product, first.idlocation)
            .await
            .unwrap_err();
        assert_eq!(
            refused.as_api().unwrap().code,
            Some(Reported::Known(picqer_rs::ErrorCode::WrongStatus))
        );
        let in_use = locations.delete(first.idlocation).await.unwrap_err();
        assert_eq!(in_use.as_api().unwrap().status, 400);
        let on_location = locations.products_all(first.idlocation).await?;
        assert_eq!(on_location.len(), 1);
        assert_eq!(on_location[0].idproduct, product);
        assert!(on_location[0].intent.is_none());

        products
            .move_stock(
                product,
                &StockMove::new(
                    one,
                    StockPlace::Location(first.idlocation),
                    StockPlace::Location(container.idlocation),
                ),
            )
            .await?;
        let in_container = locations.products_all(container.idlocation).await?;
        assert_eq!(in_container.len(), 1);
        assert!(in_container[0].intent.is_some());
        let intent = locations
            .set_intent(container.idlocation, product, Some(second.idlocation))
            .await?;
        assert_eq!(intent.idlocation, Some(second.idlocation));
        let in_container = locations.products_all(container.idlocation).await?;
        assert_eq!(
            in_container[0].intent.as_ref().and_then(|i| i.idlocation),
            Some(second.idlocation)
        );
        let cleared = locations
            .set_intent(container.idlocation, product, None)
            .await?;
        assert_eq!(cleared.idlocation, None);
        let not_a_container = locations
            .set_intent(first.idlocation, product, Some(second.idlocation))
            .await
            .unwrap_err();
        assert_eq!(not_a_container.as_api().unwrap().status, 400);

        let tracked = products
            .update(
                product,
                &ProductChanges::new()
                    .track_lot_numbers(true)
                    .track_expiration_dates(true),
            )
            .await?;
        assert_eq!(tracked.track_lot_numbers, Some(true));
        assert_eq!(tracked.track_expiration_dates, Some(true));
        let corrected = products
            .update_location(
                product,
                second.idlocation,
                &ProductLocationChanges::new()
                    .lot_number(LotNumber::new("LOT-2026-001").unwrap())
                    .expiration_date("2027-01-15".parse().unwrap()),
            )
            .await?;
        assert_eq!(corrected.lot_number.as_deref(), Some("LOT-2026-001"));
        assert_eq!(
            corrected.expiration_date,
            Some("2027-01-15".parse().unwrap())
        );
        let stored = products.locations(product).await?;
        let at_second = stored
            .iter()
            .find(|entry| entry.location.idlocation == second.idlocation)
            .unwrap();
        assert_eq!(at_second.lot_number.as_deref(), Some("LOT-2026-001"));
        assert_eq!(
            at_second.expiration_date,
            Some("2027-01-15".parse().unwrap())
        );

        let without_reason = products
            .move_stock(
                product,
                &StockMove::new(
                    one,
                    StockPlace::Location(second.idlocation),
                    StockPlace::Location(third.idlocation),
                ),
            )
            .await
            .unwrap_err();
        assert_eq!(
            without_reason.as_api().unwrap().code,
            Some(Reported::Known(picqer_rs::ErrorCode::MissingParameters))
        );
        let across = products
            .move_stock(
                product,
                &StockMove::new(
                    one,
                    StockPlace::Location(second.idlocation),
                    StockPlace::Location(third.idlocation),
                )
                .reason(reason()),
            )
            .await?;
        assert_eq!(across.from.warehouse.level.idwarehouse, warehouse);
        assert_eq!(across.to.warehouse.level.idwarehouse, other_warehouse);
        assert_eq!(across.to.warehouse.level.stock, 1);
        let levels = products.stock(product).await?;
        assert_eq!(
            levels
                .iter()
                .find(|level| level.idwarehouse == other_warehouse)
                .map(|level| level.stock),
            Some(1)
        );
        Ok::<(), picqer_rs::Error>(())
    }
    .await;

    clean_up().await;
    result.unwrap();
    assert!(products.locations(product).await.unwrap().is_empty());
    for name in names {
        assert!(
            locations
                .list_all(&LocationQuery::new().name(name))
                .await
                .unwrap()
                .is_empty()
        );
    }
}

#[tokio::test]
#[ignore = "needs a Picqer account"]
async fn a_product_can_be_received_reverted_and_the_receipt_deleted() {
    use picqer_rs::locations::{LocationName, LocationQuery, NewLocation};
    use picqer_rs::products::{LotNumber, ProductChanges};
    use picqer_rs::purchase_orders::{
        NewPurchaseOrder, NewPurchaseOrderProduct, PurchaseOrderQuery, PurchaseOrderStatus,
    };
    use picqer_rs::receipts::{
        NewReceipt, ReceiptQuery, ReceiptStatus, ReceiveProduct, ReceiveStrategy, StrategyKind,
    };
    use std::num::NonZeroU32;

    let _only_test_with_locations = LOCATION_TESTS.lock().await;
    let client = client();
    let products = client.products();
    let locations = client.locations();
    let receipts = client.receipts();
    let warehouses = client.warehouses().list_all().await.unwrap();
    let warehouse = warehouses[0].idwarehouse;
    let reason = || StockReason::new("picqer-rs live test").unwrap();
    let code = ProductCode::new("picqer-rs-live-locations").unwrap();
    let product = match products.find_by_productcode(&code).await.unwrap() {
        Some(existing) => {
            products.activate(existing.idproduct).await.unwrap();
            existing.idproduct
        }
        None => {
            let vat_group = client.vat_groups().list_all().await.unwrap()[0].idvatgroup;
            products
                .create(&NewProduct::new(
                    vat_group,
                    code.clone(),
                    ProductName::new("picqer-rs live test locations").unwrap(),
                    Money::from_cents(100).unwrap(),
                ))
                .await
                .unwrap()
                .idproduct
        }
    };
    let reference = "picqer-rs-live-receipt";
    let existing = client
        .purchase_orders()
        .list_all(&PurchaseOrderQuery::new().status(PurchaseOrderStatus::Purchased))
        .await
        .unwrap()
        .into_iter()
        .find(|order| order.supplier_orderid.as_deref() == Some(reference));
    let order = match existing {
        Some(order) => order,
        None => {
            let supplier = client.suppliers().list_all().await.unwrap()[0].idsupplier;
            let made = client
                .purchase_orders()
                .create(
                    &NewPurchaseOrder::for_supplier(supplier)
                        .supplier_orderid(reference)
                        .product(NewPurchaseOrderProduct::new(
                            product,
                            NonZeroU32::new(20).unwrap(),
                        )),
                )
                .await
                .unwrap();
            client
                .purchase_orders()
                .mark_as_purchased(made.idpurchaseorder)
                .await
                .unwrap()
        }
    };
    let purchase_order = order.idpurchaseorder;
    let names = ["ZZP-rc2", "ZZP-rc3"];
    let clean_up = || async {
        for old in receipts
            .list_all(
                &ReceiptQuery::new()
                    .idpurchaseorder(purchase_order)
                    .status(ReceiptStatus::Processing),
            )
            .await
            .unwrap()
        {
            let full = receipts.get(old.idreceipt).await.unwrap();
            for line in full
                .products
                .iter()
                .filter(|line| line.reverted_at.is_none())
            {
                receipts
                    .revert(old.idreceipt, line.idreceipt_product)
                    .await
                    .unwrap();
            }
            receipts.delete(old.idreceipt).await.unwrap();
        }
        products
            .update(
                product,
                &ProductChanges::new()
                    .track_lot_numbers(false)
                    .track_expiration_dates(false),
            )
            .await
            .unwrap();
        for entry in products.locations(product).await.unwrap() {
            products
                .unlink_location(product, entry.location.idlocation)
                .await
                .ok();
        }
        for name in names {
            for old in locations
                .list_all(&LocationQuery::new().name(name))
                .await
                .unwrap()
            {
                locations.delete(old.idlocation).await.unwrap();
            }
        }
        for level in warehouses.iter() {
            products
                .update_stock(product, level.idwarehouse, &StockUpdate::set(0, reason()))
                .await
                .unwrap();
        }
    };
    clean_up().await;
    assert_eq!(
        order.status,
        Reported::Known(PurchaseOrderStatus::Purchased)
    );

    let first = locations
        .create(&NewLocation::new(
            LocationName::new(names[0]).unwrap(),
            warehouse,
        ))
        .await
        .unwrap();
    let second = locations
        .create(&NewLocation::new(
            LocationName::new(names[1]).unwrap(),
            warehouse,
        ))
        .await
        .unwrap();
    let receipt = receipts
        .create(NewReceipt::for_purchase_order(purchase_order))
        .await
        .unwrap();
    let id = receipt.idreceipt;

    let result = async {
        assert_eq!(receipt.version, Some(2));
        assert_eq!(
            receipt.status,
            Some(Reported::Known(ReceiptStatus::Processing))
        );
        assert_eq!(receipt.amount_received, Some(0));
        assert!(receipt.products.is_empty());
        assert_eq!(
            receipt.purchaseorder.as_ref().map(|po| po.idpurchaseorder),
            Some(purchase_order)
        );
        assert_eq!(receipts.get(id).await?, receipt);
        let listed = receipts
            .list_all(&ReceiptQuery::new().idpurchaseorder(purchase_order))
            .await?;
        assert!(listed.iter().any(|found| found.idreceipt == id));
        assert!(
            receipts
                .list_all(&ReceiptQuery::new().receiptid(receipt.receiptid.clone().unwrap()))
                .await?
                .iter()
                .all(|found| found.idreceipt == id)
        );

        let expected = receipts.expected_products(id).await?;
        let wanted = expected
            .iter()
            .find(|entry| entry.product.idproduct == product)
            .expect("the product is expected");
        assert_eq!(wanted.totals.as_ref().unwrap().expected, Some(20));
        assert_eq!(receipts.expected_product(id, product).await?, *wanted);
        let line = wanted
            .strategies
            .iter()
            .find(|strategy| {
                strategy.strategy == Reported::Known(StrategyKind::PurchaseorderProduct)
            })
            .expect("a strategy for a line of the purchase order")
            .parameters
            .as_ref()
            .and_then(|parameters| parameters.idpurchaseorder_product)
            .unwrap();
        assert!(
            wanted
                .strategies
                .iter()
                .any(|strategy| strategy.strategy == Reported::Known(StrategyKind::Automatic))
        );
        assert!(
            wanted
                .strategies
                .iter()
                .any(|strategy| strategy.strategy == Reported::Known(StrategyKind::New))
        );

        let two = NonZeroU32::new(2).unwrap();
        let one = NonZeroU32::new(1).unwrap();
        let by_line = receipts
            .receive(
                id,
                &ReceiveProduct::new(product, two, ReceiveStrategy::PurchaseOrderProduct(line)),
            )
            .await?;
        assert_eq!(by_line.len(), 1);
        assert_eq!(by_line[0].amount, Some(2));
        assert_eq!(by_line[0].location, None);
        let automatic = receipts
            .receive(
                id,
                &ReceiveProduct::new(product, one, ReceiveStrategy::Automatic)
                    .idlocation(first.idlocation),
            )
            .await?;
        assert_eq!(
            automatic[0].location.as_ref().unwrap().idlocation,
            first.idlocation
        );
        let added = receipts
            .receive(id, &ReceiveProduct::new(product, one, ReceiveStrategy::New))
            .await?;
        assert_eq!(added.len(), 1);

        let after = receipts.expected_product(id, product).await?;
        assert_eq!(after.totals.as_ref().unwrap().received, Some(4));
        let stored = receipts.get(id).await?;
        assert_eq!(stored.amount_received, Some(4));
        assert_eq!(stored.products.len(), 3);
        let level = |warehouse| async move {
            products.stock(product).await.map(|levels| {
                levels
                    .iter()
                    .find(|level| level.idwarehouse == warehouse)
                    .map(|level| level.stock)
            })
        };
        assert_eq!(level(warehouse).await?, Some(4));

        products
            .update(
                product,
                &ProductChanges::new()
                    .track_lot_numbers(true)
                    .track_expiration_dates(true),
            )
            .await?;
        let date = "2027-01-15".parse().unwrap();
        let lot = |text: &str| LotNumber::new(text).unwrap();
        let bare = ReceiveProduct::new(product, one, ReceiveStrategy::New);
        let without_lot = receipts
            .receive(id, &bare.clone().idlocation(second.idlocation))
            .await
            .unwrap_err();
        assert_eq!(
            without_lot.as_api().unwrap().code,
            Some(Reported::Known(picqer_rs::ErrorCode::BadRequest))
        );
        let without_location = receipts
            .receive(
                id,
                &bare.clone().lot_number(lot("L1")).expiration_date(date),
            )
            .await
            .unwrap_err();
        assert_eq!(without_location.as_api().unwrap().status, 400);
        let traced = receipts
            .receive(
                id,
                &bare
                    .clone()
                    .idlocation(second.idlocation)
                    .lot_number(lot("L1"))
                    .expiration_date(date),
            )
            .await?;
        assert_eq!(traced[0].lot_number.as_deref(), Some("L1"));
        assert_eq!(traced[0].expiration_date, Some(date));
        let other_lot = receipts
            .receive(
                id,
                &bare
                    .clone()
                    .idlocation(second.idlocation)
                    .lot_number(lot("L2"))
                    .expiration_date(date),
            )
            .await
            .unwrap_err();
        assert_eq!(
            other_lot.as_api().unwrap().code,
            Some(Reported::Known(picqer_rs::ErrorCode::InvalidLocation))
        );

        let full = receipts.get(id).await?;
        assert_eq!(full.products.len(), 4);
        for line in full
            .products
            .iter()
            .filter(|line| line.reverted_at.is_none())
        {
            let reverted = receipts.revert(id, line.idreceipt_product).await?;
            assert!(reverted.reverted_at.is_some());
            assert!(reverted.reverted_by_iduser.is_some());
        }
        let after_revert = receipts.get(id).await?;
        assert_eq!(after_revert.amount_received, Some(0));
        assert_eq!(level(warehouse).await?, Some(0));
        assert_eq!(
            client.purchase_orders().get(purchase_order).await?.status,
            Reported::Known(PurchaseOrderStatus::Purchased)
        );

        let empty = receipts
            .create(NewReceipt::for_purchase_order(purchase_order))
            .await?;
        let completed = receipts.complete(empty.idreceipt).await?;
        assert_eq!(
            completed.status,
            Some(Reported::Known(ReceiptStatus::Completed))
        );
        let late = receipts
            .receive(
                empty.idreceipt,
                &ReceiveProduct::new(product, one, ReceiveStrategy::New),
            )
            .await
            .unwrap_err();
        assert_eq!(
            late.as_api().unwrap().code,
            Some(Reported::Known(picqer_rs::ErrorCode::WrongStatus))
        );
        receipts.delete(empty.idreceipt).await?;
        Ok::<(), picqer_rs::Error>(())
    }
    .await;

    clean_up().await;
    result.unwrap();
    assert!(receipts.get(id).await.unwrap_err().is_not_found());
}

#[tokio::test]
#[ignore = "needs a Picqer account"]
async fn comments_can_be_made_on_a_picklist_batch_and_a_receipt() {
    use picqer_rs::comments::{CommentBody, CommentSource, CommentTarget, NewComment};
    use picqer_rs::picklist_batches::BatchQuery;
    use picqer_rs::receipts::ReceiptQuery;

    let client = client();
    let batch = client
        .picklist_batches()
        .list(&BatchQuery::new(), picqer_rs::pagination::Page::FIRST)
        .await
        .unwrap()
        .into_iter()
        .next()
        .expect("the account has a picklist batch")
        .idpicklist_batch;
    let receipt = client
        .receipts()
        .list(&ReceiptQuery::new(), picqer_rs::pagination::Page::FIRST)
        .await
        .unwrap()
        .into_iter()
        .next()
        .expect("the account has a receipt")
        .idreceipt;

    for target in [
        CommentTarget::PicklistBatch(batch),
        CommentTarget::Receipt(receipt),
    ] {
        let comment = client
            .comments()
            .add(
                target,
                &NewComment::new(CommentBody::new("picqer-rs live test").unwrap()),
            )
            .await
            .unwrap();
        let result = async {
            match (&comment.source, target) {
                (
                    CommentSource::PicklistBatch {
                        idpicklist_batch, ..
                    },
                    CommentTarget::PicklistBatch(id),
                ) => assert_eq!(*idpicklist_batch, id),
                (CommentSource::Receipt { idreceipt, .. }, CommentTarget::Receipt(id)) => {
                    assert_eq!(*idreceipt, id)
                }
                (source, target) => panic!("{source:?} is not the source of {target:?}"),
            }
            assert!(
                client
                    .comments()
                    .for_target_all(target)
                    .await?
                    .contains(&comment)
            );
            assert_eq!(client.comments().get(comment.idcomment).await?, comment);
            Ok::<(), picqer_rs::Error>(())
        }
        .await;
        client.comments().delete(comment.idcomment).await.unwrap();
        result.unwrap();
    }
}

#[tokio::test]
#[ignore = "needs a Picqer account"]
async fn a_return_can_be_made_changed_received_and_deleted() {
    use picqer_rs::country::CountryCode;
    use picqer_rs::customers::Language;
    use picqer_rs::returns::{
        NewReplacementProduct, NewReturn, NewReturnLog, NewReturnedProduct, ReceiveReturn,
        ReceivedProduct, ReplacementProductChanges, ReturnChanges, ReturnLogType, ReturnQuery,
        ReturnStatusFilter, ReturnedProductChanges, ReturnedProductStatus,
    };
    use picqer_rs::types::{EmailAddress, NonEmpty};
    use std::num::NonZeroU32;

    let client = client();
    let returns = client.returns();
    let statuses = client.return_statuses().list().await.unwrap();
    let default_status = statuses
        .iter()
        .find(|status| status.default == Some(true))
        .expect("a default return status")
        .idreturn_status;
    let other_status = statuses
        .iter()
        .find(|status| {
            status.default != Some(true)
                && status.completed != Some(true)
                && status.deleted_at.is_none()
        })
        .expect("another return status that does not complete")
        .idreturn_status;
    let completed_status = statuses
        .iter()
        .find(|status| status.completed == Some(true) && status.deleted_at.is_none())
        .expect("a return status that completes")
        .idreturn_status;
    let reasons = client.return_reasons().list().await.unwrap();
    let reason = reasons[0].idreturn_reason;
    let other_reason = reasons[1].idreturn_reason;
    let product = client
        .products()
        .list_all(&ProductQuery::new())
        .await
        .unwrap()
        .into_iter()
        .next()
        .unwrap()
        .idproduct;

    let reference = "picqer-rs-live-return";
    for old in returns
        .list_all(&ReturnQuery::new().search(reference))
        .await
        .unwrap()
    {
        returns.delete(old.idreturn).await.unwrap();
    }
    let amount = |amount| NonZeroU32::new(amount).unwrap();
    let made = returns
        .create(
            &NewReturn::new(default_status)
                .name("picqer-rs live test")
                .reference(reference)
                .country(CountryCode::NL)
                .emailaddress(EmailAddress::new("live-test@example.com").unwrap())
                .language(Language::Nl)
                .returned_product(
                    NewReturnedProduct::new(reason, product, amount(2))
                        .price(Money::from_cents(1250).unwrap()),
                )
                .replacement_product(NewReplacementProduct::new(product, amount(1))),
        )
        .await
        .unwrap();
    let id = made.idreturn;

    let result = async {
        assert_eq!(made.idreturn_status, default_status);
        assert_eq!(made.reference.as_deref(), Some(reference));
        assert_eq!(made.country, Some(Reported::Known(CountryCode::NL)));
        assert_eq!(made.language, Some(Reported::Known(Language::Nl)));
        assert_eq!(made.emailaddress.as_deref(), Some("live-test@example.com"));
        let stored = returns.get(id).await?;
        assert_eq!(stored.returned_products.len(), 1);
        assert_eq!(stored.replacement_products.len(), 1);
        let line = &stored.returned_products[0];
        assert_eq!(line.amount, Some(2));
        assert_eq!(line.price, Some(Money::from_cents(1250).unwrap()));
        assert_eq!(
            line.status,
            Some(Reported::Known(ReturnedProductStatus::Expected))
        );
        assert_eq!(line.changeable, Some(true));
        assert_eq!(line.idreturn_reason, Some(reason));
        assert_eq!(
            stored.replacement_products[0].price,
            Some(Money::from_cents(0).unwrap())
        );
        let totals = stored.totals.as_ref().unwrap();
        assert_eq!(totals.returned_products_count, Some(2));
        assert_eq!(totals.replacement_products_count, Some(1));
        assert_eq!(
            stored.return_status.as_ref().unwrap().idreturn_status,
            default_status
        );

        assert!(
            returns
                .list_all(&ReturnQuery::new().search(reference))
                .await?
                .iter()
                .any(|found| found.idreturn == id)
        );
        assert!(
            returns
                .list_all(&ReturnQuery::new().status(ReturnStatusFilter::Status(default_status)))
                .await?
                .iter()
                .any(|found| found.idreturn == id)
        );
        assert!(
            returns
                .list_all(&ReturnQuery::new().status(ReturnStatusFilter::Processing))
                .await?
                .iter()
                .any(|found| found.idreturn == id)
        );
        assert!(
            !returns
                .list_all(&ReturnQuery::new().status(ReturnStatusFilter::Completed))
                .await?
                .iter()
                .any(|found| found.idreturn == id)
        );
        assert!(
            returns
                .list_all(&ReturnQuery::new().emailaddress("live-test@example.com"))
                .await?
                .iter()
                .any(|found| found.idreturn == id)
        );

        let changed = returns
            .update(
                id,
                &ReturnChanges::new()
                    .without_name()
                    .tracking_code("TRK-live")
                    .contactname("Live Test")
                    .received_at("2026-11-09 10:00:00".parse().unwrap())
                    .idreturn_status(other_status),
            )
            .await?;
        assert_eq!(changed.name, None);
        assert_eq!(changed.tracking_code.as_deref(), Some("TRK-live"));
        assert_eq!(changed.contactname.as_deref(), Some("Live Test"));
        assert_eq!(changed.idreturn_status, other_status);
        assert_eq!(
            changed.received_at,
            Some("2026-11-09 10:00:00".parse().unwrap())
        );
        let cleared = returns
            .update(id, &ReturnChanges::new().without_tracking_code())
            .await?;
        assert_eq!(cleared.tracking_code, None);

        let added = returns
            .add_returned_product(
                id,
                NewReturnedProduct::new(other_reason, product, amount(1))
                    .price(Money::from_cents(1295).unwrap()),
            )
            .await?;
        assert_eq!(added.price, Some(Money::from_cents(1295).unwrap()));
        let updated = returns
            .update_returned_product(
                id,
                added.idreturn_product,
                ReturnedProductChanges::new()
                    .price(Money::from_cents(2000).unwrap())
                    .amount(amount(3))
                    .idreturn_reason(reason),
            )
            .await?;
        assert_eq!(updated.amount, Some(3));
        assert_eq!(updated.price, Some(Money::from_cents(2000).unwrap()));
        assert_eq!(updated.idreturn_reason, Some(reason));
        assert_eq!(returns.returned_products(id).await?.len(), 2);
        returns
            .remove_returned_product(id, added.idreturn_product)
            .await?;
        assert_eq!(returns.returned_products(id).await?.len(), 1);

        let replacement = returns
            .add_replacement_product(
                id,
                NewReplacementProduct::new(product, amount(2))
                    .price(Money::from_cents(950).unwrap()),
            )
            .await?;
        assert_eq!(replacement.price, Some(Money::from_cents(950).unwrap()));
        assert_eq!(replacement.status.as_deref(), Some("concept"));
        let changed_replacement = returns
            .update_replacement_product(
                id,
                replacement.idreturn_product_replacement,
                ReplacementProductChanges::new()
                    .price(Money::from_cents(500).unwrap())
                    .amount(amount(3)),
            )
            .await?;
        assert_eq!(changed_replacement.amount, Some(3));
        assert_eq!(returns.replacement_products(id).await?.len(), 2);
        returns
            .remove_replacement_product(id, replacement.idreturn_product_replacement)
            .await?;
        assert_eq!(returns.replacement_products(id).await?.len(), 1);

        let comment = returns
            .add_log(id, &NewReturnLog::comment("picqer-rs live comment"))
            .await?;
        assert_eq!(
            comment.log_type,
            Some(Reported::Known(ReturnLogType::Comment))
        );
        assert_eq!(comment.notified_customer, Some(false));
        let status_change = returns
            .add_log(id, &NewReturnLog::status(default_status).message("back"))
            .await?;
        assert_eq!(
            status_change.log_type,
            Some(Reported::Known(ReturnLogType::StatusChanged))
        );
        assert_eq!(returns.get(id).await?.idreturn_status, default_status);
        let completion = returns
            .add_log(id, &NewReturnLog::status(completed_status))
            .await?;
        assert_eq!(
            completion.log_type,
            Some(Reported::Known(ReturnLogType::Completed))
        );
        let completed = returns.get(id).await?;
        assert!(completed.completed_at.is_some());
        assert!(completed.completed_by_iduser.is_some());
        assert!(
            returns
                .list_all(&ReturnQuery::new().status(ReturnStatusFilter::Completed))
                .await?
                .iter()
                .any(|found| found.idreturn == id)
        );
        returns
            .add_log(id, &NewReturnLog::status(default_status))
            .await?;
        let logs = returns.logs(id).await?;
        assert!(
            logs.iter()
                .any(|log| log.log_type == Some(Reported::Known(ReturnLogType::Created)))
        );
        assert!(
            logs.iter()
                .any(|log| log.message.as_deref() == Some("picqer-rs live comment"))
        );

        let line = returns.get(id).await?.returned_products[0].idreturn_product;
        for status in [
            ReturnedProductStatus::Received,
            ReturnedProductStatus::Unsalable,
            ReturnedProductStatus::Expected,
        ] {
            let received = returns
                .receive(
                    id,
                    &ReceiveReturn::new(
                        NonEmpty::new(vec![ReceivedProduct::new(line, status)]).unwrap(),
                    ),
                )
                .await?;
            assert_eq!(received.idreturn, id);
            let now = returns.returned_products(id).await?;
            assert_eq!(now[0].status, Some(Reported::Known(status)));
            assert_eq!(now[0].changeable, Some(true));
        }
        let adds_stock = ReceiveReturn::new(
            NonEmpty::new(vec![ReceivedProduct::new(
                line,
                ReturnedProductStatus::AddedToStock,
            )])
            .unwrap(),
        );
        assert!(matches!(
            returns.receive(id, &adds_stock).await.unwrap_err(),
            picqer_rs::Error::Validation(_)
        ));
        assert!(
            returns
                .logs(id)
                .await?
                .iter()
                .any(|log| log.log_type == Some(Reported::Known(ReturnLogType::ReceivedProducts)))
        );
        Ok::<(), picqer_rs::Error>(())
    }
    .await;

    returns.delete(id).await.unwrap();
    result.unwrap();
    assert!(returns.get(id).await.unwrap_err().is_not_found());
}

#[tokio::test]
#[ignore = "needs a Picqer account, takes a minute"]
async fn a_backorder_can_be_read_deleted_and_processed() {
    use picqer_rs::backorders::BackorderQuery;
    use picqer_rs::customers::CustomerQuery;
    use picqer_rs::orders::{NewOrder, NewOrderProduct, OrderQuery, OrderStatus};
    use picqer_rs::types::NonEmpty;
    use std::num::NonZeroU32;

    let client = client();
    let backorders = client.backorders();
    let reference = "picqer-rs-live-backorder";
    let customer = client
        .customers()
        .list_all(&CustomerQuery::new())
        .await
        .unwrap()
        .into_iter()
        .next()
        .unwrap();

    for old in client
        .orders()
        .list_all(
            &OrderQuery::new()
                .reference(reference)
                .status(OrderStatus::Processing),
        )
        .await
        .unwrap()
    {
        client.orders().force_cancel(old.idorder).await.unwrap();
        wait_for_status(&client, old.idorder, &[OrderStatus::Cancelled], 90).await;
    }
    let mut existing = None;
    for status in [OrderStatus::Concept, OrderStatus::Cancelled] {
        if let Some(order) = client
            .orders()
            .list_all(&OrderQuery::new().reference(reference).status(status))
            .await
            .unwrap()
            .into_iter()
            .next()
        {
            if status == OrderStatus::Cancelled {
                client
                    .orders()
                    .undo_cancellation(order.idorder)
                    .await
                    .unwrap();
            }
            existing =
                Some(wait_for_status(&client, order.idorder, &[OrderStatus::Concept], 60).await);
            break;
        }
    }
    let order = match existing {
        Some(order) => order,
        None => {
            let product = client
                .products()
                .list_all(&ProductQuery::new())
                .await
                .unwrap()
                .into_iter()
                .find(|product| {
                    product.idfulfilment_customer == customer.idfulfilment_customer
                        && !product.stock.is_empty()
                        && product.stock.iter().all(|level| level.freestock <= 0)
                })
                .expect("a product of the customer without free stock");
            let line = NewOrderProduct::by_id(product.idproduct, NonZeroU32::new(1).unwrap());
            client
                .orders()
                .create(
                    &NewOrder::for_customer(
                        customer.idcustomer,
                        NonEmpty::new(vec![line]).unwrap(),
                    )
                    .reference(reference),
                )
                .await
                .unwrap()
        }
    };
    let id = order.idorder;
    let product = order.products[0].idproduct;
    macro_rules! ensure {
        ($condition:expr) => {
            if !$condition {
                return Err(picqer_rs::Error::UnexpectedResponse(stringify!($condition)));
            }
        };
    }
    let backorder_of = || async {
        for _ in 0..30 {
            let found = backorders.for_order(id).await?;
            if let Some(first) = found.into_iter().next() {
                return Ok::<_, picqer_rs::Error>(first);
            }
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
        Err(picqer_rs::Error::UnexpectedResponse(
            "the order made no backorder",
        ))
    };

    let mut restore = None;
    let result = async {
        client.orders().process(id).await?;
        wait_for_status(&client, id, &[OrderStatus::Processing], 150).await;
        let made = backorder_of().await?;
        ensure!((made.idorder) == (id));
        ensure!((made.idproduct) == (product));
        ensure!((made.idcustomer) == (Some(customer.idcustomer)));
        ensure!((made.amount) == (1));
        ensure!((made.amount_available) == (Some(0)));
        ensure!(made.idreturn.is_none());
        ensure!((made.has_parts) == (Some(false)));
        ensure!(made.created_at.is_some());
        ensure!((made.is_purchased) == (Some(false)));

        ensure!((backorders.get(made.idbackorder).await?) == (made));
        ensure!(
            backorders
                .list_all(&BackorderQuery::new().idorder(id))
                .await?
                .iter()
                .any(|found| found.idbackorder == made.idbackorder)
        );
        ensure!(
            backorders
                .list_all(
                    &BackorderQuery::new()
                        .idproduct(product)
                        .idcustomer(customer.idcustomer)
                )
                .await?
                .iter()
                .any(|found| found.idbackorder == made.idbackorder)
        );
        ensure!(
            backorders
                .list_all(&BackorderQuery::new().since_id(made.idbackorder))
                .await?
                .iter()
                .all(|found| found.idbackorder.0 >= made.idbackorder.0)
        );
        ensure!(
            backorders
                .list_all(&BackorderQuery::new().since("2030-01-01 00:00:00".parse().unwrap()))
                .await?
                .is_empty()
        );

        backorders.delete(made.idbackorder).await?;
        ensure!(
            backorders
                .get(made.idbackorder)
                .await
                .unwrap_err()
                .is_not_found()
        );
        ensure!(backorders.for_order(id).await?.is_empty());
        let cancelled = wait_for_status(&client, id, &[OrderStatus::Cancelled], 90).await;
        ensure!((cancelled.products[0].amount_cancelled) == (Some(1)));

        client.orders().undo_cancellation(id).await?;
        wait_for_status(&client, id, &[OrderStatus::Concept], 60).await;
        client.orders().process(id).await?;
        wait_for_status(&client, id, &[OrderStatus::Processing], 150).await;
        let again = backorder_of().await?;
        ensure!(again.idbackorder != made.idbackorder);

        let warehouse = again.idwarehouse.expect("the backorder has a warehouse");
        let before = client
            .products()
            .stock_in_warehouse(product, warehouse)
            .await?;
        restore = Some((warehouse, u32::try_from(before.level.stock).unwrap_or(0)));
        client
            .products()
            .update_stock(
                product,
                warehouse,
                &StockUpdate::adjust(
                    StockTarget::NoSpecificLocation,
                    std::num::NonZeroI64::new(1).unwrap(),
                    StockReason::new("picqer-rs live test").unwrap(),
                ),
            )
            .await?;
        for _ in 0..20 {
            if backorders.get(again.idbackorder).await?.amount_available == Some(1) {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
        ensure!((backorders.get(again.idbackorder).await?.amount_available) == (Some(1)));

        backorders.process().await?;
        let mut picklists = Vec::new();
        for _ in 0..30 {
            picklists = client.orders().get(id).await?.picklists;
            if !picklists.is_empty() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
        ensure!(!picklists.is_empty());
        let mut left = backorders.for_order(id).await?;
        for _ in 0..20 {
            if left.is_empty() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            left = backorders.for_order(id).await?;
        }
        ensure!(left.is_empty());
        Ok::<(), picqer_rs::Error>(())
    }
    .await;

    client.orders().force_cancel(id).await.unwrap();
    wait_for_status(&client, id, &[OrderStatus::Cancelled], 90).await;
    if let Some((warehouse, amount)) = restore {
        client
            .products()
            .update_stock(
                product,
                warehouse,
                &StockUpdate::set(amount, StockReason::new("picqer-rs live test").unwrap()),
            )
            .await
            .unwrap();
    }
    result.unwrap();
}

#[tokio::test]
#[ignore = "needs a Picqer account"]
async fn a_task_can_be_made_changed_completed_and_deleted() {
    use picqer_rs::tasks::{NewTask, TaskChanges, TaskQuery, TaskTitle};

    let client = client();
    let tasks = client.tasks();
    let me = client.users().me().await.unwrap().iduser;
    let prefix = "picqer-rs-live-task";
    for old in tasks.list_all(TaskQuery::new()).await.unwrap() {
        if old.title.starts_with(prefix) {
            tasks.delete(old.idtask).await.unwrap();
        }
    }
    let date = |text: &str| text.parse::<picqer_rs::types::Date>().unwrap();
    let time = |text: &str| text.parse::<picqer_rs::tasks::TaskTime>().unwrap();

    let made = tasks
        .create(
            &NewTask::new(TaskTitle::new(format!("{prefix} one")).unwrap())
                .description("live test")
                .deadline(date("2031-03-04"), time("23:59:00"))
                .assigned_to(me),
        )
        .await
        .unwrap();
    let plain = tasks
        .create(&NewTask::new(
            TaskTitle::new(format!("{prefix} two")).unwrap(),
        ))
        .await
        .unwrap();
    let result = async {
        assert_eq!(made.description.as_deref(), Some("live test"));
        assert_eq!(made.assigned_to_iduser, Some(me));
        assert_eq!(made.is_seen_by_assignee, Some(false));
        assert_eq!(
            made.deadline_date.unwrap().to_string(),
            "2031-03-04 00:00:00"
        );
        assert_eq!(made.deadline_time, Some(time("23:59:00")));
        assert_eq!(made.created_by_iduser, Some(me));
        assert_eq!(made.completed_at, None);
        assert_eq!(plain.assigned_to_iduser, None);
        assert_eq!(plain.is_seen_by_assignee, None);
        assert_eq!(plain.deadline_date, None);
        assert_eq!(tasks.get(made.idtask).await?, made);

        let renamed = tasks
            .update(
                made.idtask,
                &TaskChanges::new().title(TaskTitle::new(format!("{prefix} renamed")).unwrap()),
            )
            .await?;
        assert_eq!(renamed.title, format!("{prefix} renamed"));
        assert_eq!(renamed.assigned_to_iduser, Some(me));
        assert_eq!(renamed.description.as_deref(), Some("live test"));

        let moved = tasks
            .update(
                made.idtask,
                &TaskChanges::new().deadline(date("2031-03-05"), time("09:30:00")),
            )
            .await?;
        assert_eq!(
            moved.deadline_date.unwrap().to_string(),
            "2031-03-05 00:00:00"
        );
        assert_eq!(moved.deadline_time, Some(time("09:30:00")));
        let without_time = tasks
            .update(made.idtask, &TaskChanges::new().without_deadline_time())
            .await?;
        assert_eq!(without_time.deadline_time, None);
        assert!(without_time.deadline_date.is_some());
        let cleared = tasks
            .update(made.idtask, &TaskChanges::new().without_description())
            .await?;
        assert_eq!(cleared.description, None);
        assert_eq!(cleared.assigned_to_iduser, Some(me));

        let unassigned = tasks
            .update(made.idtask, &TaskChanges::new().unassigned())
            .await?;
        assert_eq!(unassigned.assigned_to_iduser, None);
        let assigned = tasks
            .update(plain.idtask, &TaskChanges::new().assigned_to(me))
            .await?;
        assert_eq!(assigned.assigned_to_iduser, Some(me));

        let done = tasks.complete(made.idtask).await?;
        assert!(done.completed_at.is_some());
        assert_eq!(done.completed_by_iduser, Some(me));
        assert_eq!(
            tasks.complete(made.idtask).await?.completed_by_iduser,
            Some(me)
        );
        let completed = tasks.list_all(TaskQuery::new().completed(true)).await?;
        assert!(completed.iter().any(|task| task.idtask == made.idtask));
        assert!(!completed.iter().any(|task| task.idtask == plain.idtask));
        let open = tasks.list_all(TaskQuery::new().completed(false)).await?;
        assert!(open.iter().any(|task| task.idtask == plain.idtask));
        assert!(!open.iter().any(|task| task.idtask == made.idtask));
        let reopened = tasks.uncomplete(made.idtask).await?;
        assert_eq!(reopened.completed_at, None);
        assert_eq!(reopened.completed_by_iduser, None);
        assert!(
            tasks
                .list_all(TaskQuery::new())
                .await?
                .iter()
                .any(|task| task.idtask == made.idtask)
        );
        Ok::<(), picqer_rs::Error>(())
    }
    .await;
    tasks.delete(made.idtask).await.unwrap();
    tasks.delete(plain.idtask).await.unwrap();
    result.unwrap();
    let gone = tasks.get(made.idtask).await.unwrap_err();
    assert!(gone.is_not_found());
    assert!(gone.as_api().unwrap().errors.is_some());
}

#[tokio::test]
#[ignore = "needs a Picqer account"]
async fn a_picking_container_can_be_made_linked_and_deleted() {
    use picqer_rs::picking_containers::{PickingContainerName, PickingContainerQuery};
    use picqer_rs::picklists::PicklistQuery;

    let client = client();
    let containers = client.picking_containers();
    let name = |text: &str| PickingContainerName::new(text).unwrap();
    for old in containers
        .list_all(PickingContainerQuery::new())
        .await
        .unwrap()
    {
        if old.name.starts_with("picqer-rs-live") {
            containers.delete(old.idpicking_container).await.unwrap();
        }
    }
    let picklists = client
        .picklists()
        .list(
            &PicklistQuery::new().status(picqer_rs::picklists::PicklistStatus::New),
            picqer_rs::pagination::Page::FIRST,
        )
        .await
        .unwrap();
    let (first, second) = (picklists[0].idpicklist, picklists[1].idpicklist);

    let one = containers
        .create(&name("picqer-rs-live-one"))
        .await
        .unwrap();
    let two = containers
        .create(&name("picqer-rs-live-two"))
        .await
        .unwrap();
    let result = async {
        assert_eq!(one.name, "picqer-rs-live-one");
        assert_eq!(one.idpicklist, None);
        assert!(one.created_at.is_some());
        assert_eq!(containers.get(one.idpicking_container).await?, one);
        let duplicate = containers
            .create(&name("picqer-rs-live-one"))
            .await
            .unwrap_err();
        assert_eq!(duplicate.as_api().unwrap().status, 400);

        let renamed = containers
            .rename(one.idpicking_container, &name("picqer-rs-live-renamed"))
            .await?;
        assert_eq!(renamed.name, "picqer-rs-live-renamed");

        let linked = containers.link(one.idpicking_container, first).await?;
        assert_eq!(linked.idpicklist, Some(first));
        let found = containers
            .list_all(PickingContainerQuery::new().idpicklist(first))
            .await?;
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].idpicking_container, one.idpicking_container);

        let moved = containers.link(two.idpicking_container, first).await?;
        assert_eq!(moved.idpicklist, Some(first));
        assert_eq!(
            containers.get(one.idpicking_container).await?.idpicklist,
            None
        );
        let replaced = containers.link(two.idpicking_container, second).await?;
        assert_eq!(replaced.idpicklist, Some(second));
        assert!(
            containers
                .list_all(PickingContainerQuery::new().idpicklist(first))
                .await?
                .is_empty()
        );
        let unknown = containers
            .link(two.idpicking_container, picqer_rs::types::PicklistId(1))
            .await
            .unwrap_err();
        assert_eq!(unknown.as_api().unwrap().status, 400);
        Ok::<(), picqer_rs::Error>(())
    }
    .await;
    containers.delete(one.idpicking_container).await.unwrap();
    containers.delete(two.idpicking_container).await.unwrap();
    result.unwrap();
    assert!(
        containers
            .get(two.idpicking_container)
            .await
            .unwrap_err()
            .is_not_found()
    );
}

#[tokio::test]
#[ignore = "needs a Picqer account"]
async fn a_hook_can_be_deactivated_and_activated_again() {
    use picqer_rs::webhooks::{HookName, NewHook, WebhookEvent, WebhookSecret};
    use url::Url;

    let client = client();
    let hooks = client.hooks();
    let address = Url::parse("https://example.com/picqer-rs-live-secret").unwrap();
    let event = WebhookEvent::MovementsMoved;
    let found = hooks
        .list_all()
        .await
        .unwrap()
        .into_iter()
        .find(|hook| hook.address == address.as_str() && hook.event == Reported::Known(event));
    let hook = match found {
        Some(hook) => hook,
        None => hooks
            .create(
                &NewHook::new(
                    HookName::new("picqer-rs live hook secret").unwrap(),
                    event,
                    address.clone(),
                )
                .unwrap()
                .secret(WebhookSecret::new("my-secret-key").unwrap()),
            )
            .await
            .unwrap(),
    };
    let id = hook.idhook;

    let result = async {
        if hook.active == Some(false) {
            hooks.reactivate(id).await?;
        }
        let active = hooks.get(id).await?;
        assert_eq!(active.active, Some(true));
        assert_eq!(active.secret, Some(true));
        assert_eq!(active.event, Reported::Known(event));
        assert_eq!(active.address, address.as_str());

        let refused = hooks.reactivate(id).await.unwrap_err();
        assert_eq!(
            refused.as_api().unwrap().code,
            Some(Reported::Known(
                picqer_rs::ErrorCode::ItemNotFoundOrWrongStatus
            ))
        );
        let duplicate = hooks
            .create(
                &NewHook::new(
                    HookName::new("another name").unwrap(),
                    event,
                    address.clone(),
                )
                .unwrap(),
            )
            .await
            .unwrap_err();
        assert_eq!(
            duplicate.as_api().unwrap().code,
            Some(Reported::Known(picqer_rs::ErrorCode::MissingParameters))
        );

        hooks.deactivate(id).await?;
        assert_eq!(hooks.get(id).await?.active, Some(false));
        assert!(
            hooks
                .list_all()
                .await?
                .iter()
                .any(|listed| listed.idhook == id && listed.active == Some(false))
        );
        hooks.deactivate(id).await?;
        let back = hooks.reactivate(id).await?;
        assert_eq!(back.active, Some(true));
        Ok::<(), picqer_rs::Error>(())
    }
    .await;
    hooks.deactivate(id).await.unwrap();
    result.unwrap();
    assert_eq!(hooks.get(id).await.unwrap().active, Some(false));
    assert!(hooks.get(HookId(1)).await.unwrap_err().is_not_found());
}

#[tokio::test]
#[ignore = "needs a Picqer account"]
async fn a_tag_can_be_created_changed_and_deleted() {
    let client = client();
    let title = unique("picqer-rs-live");
    let created = client
        .tags()
        .create(&NewTag::new(
            TagTitle::new(title.clone()).unwrap(),
            HexColor::new("#823882").unwrap(),
            false,
        ))
        .await
        .unwrap();
    assert_eq!(created.title, title);

    let result = async {
        let changed = client
            .tags()
            .update(
                created.idtag,
                &TagChanges::new().color(HexColor::new("#123abc").unwrap()),
            )
            .await?;
        assert_eq!(changed.color.as_str(), "#123abc");
        assert_eq!(changed.title, title);
        Ok::<(), picqer_rs::Error>(())
    }
    .await;
    client.tags().delete(created.idtag).await.unwrap();
    result.unwrap();
    assert!(
        client
            .tags()
            .get(created.idtag)
            .await
            .unwrap_err()
            .is_not_found()
    );
}

#[tokio::test]
#[ignore = "needs a Picqer account"]
async fn a_product_can_be_created_changed_tagged_and_inactivated() {
    let client = client();
    let vat_group = client.vat_groups().list_all().await.unwrap()[0].idvatgroup;
    let tag = client.tags().list_all(&TagQuery::new()).await.unwrap()[0].idtag;
    let code = "picqer-rs-live".to_owned();
    let product_code = ProductCode::new(code.clone()).unwrap();
    let pricelist = client
        .pricelists()
        .list_all()
        .await
        .unwrap()
        .into_iter()
        .next();
    let text_field = client
        .product_fields()
        .list_all()
        .await
        .unwrap()
        .into_iter()
        .find(|field| field.field_type == Some(Reported::Known(FieldType::Text)));
    let prices: Vec<PricelistPrice> = pricelist
        .iter()
        .map(|list| PricelistPrice {
            idpricelist: list.idpricelist,
            price: Money::from_cents(199).unwrap(),
        })
        .collect();
    let values: Vec<_> = text_field
        .iter()
        .map(|field| field.value("live test").unwrap())
        .collect();

    let created = match client
        .products()
        .find_by_productcode(&product_code)
        .await
        .unwrap()
    {
        Some(existing) => {
            client
                .products()
                .activate(existing.idproduct)
                .await
                .unwrap();
            let reset = ProductChanges::new()
                .name(ProductName::new("picqer-rs live test").unwrap())
                .price(Money::from_cents(1250).unwrap())
                .weight(Grams::new(250))
                .country_of_origin(CountryCode::NL)
                .product_type(ProductType::Normal)
                .description("created by the live tests".to_owned())
                .pricelists(prices.clone())
                .productfields(values.clone());
            client
                .products()
                .update(existing.idproduct, &reset)
                .await
                .unwrap()
        }
        None => {
            let product = NewProduct::new(
                vat_group,
                product_code.clone(),
                ProductName::new("picqer-rs live test").unwrap(),
                Money::from_cents(1250).unwrap(),
            )
            .weight(Grams::new(250))
            .country_of_origin(CountryCode::NL)
            .product_type(ProductType::Normal)
            .description("created by the live tests".to_owned())
            .pricelists(prices.clone())
            .productfields(values.clone());
            client.products().create(&product).await.unwrap()
        }
    };
    assert_eq!(created.productcode, code);
    assert_eq!(created.price, Money::from_cents(1250).unwrap());
    assert_eq!(created.weight, Some(Grams::new(250)));
    assert_eq!(
        created.country_of_origin,
        Some(Reported::Known(CountryCode::NL))
    );
    assert_eq!(created.pricelists, prices);
    if let Some(field) = &text_field {
        assert!(created.productfields.iter().any(|entry| {
            entry.idproductfield == field.idproductfield
                && entry.value.as_deref() == Some("live test")
        }));
    }

    let changed = client
        .products()
        .update(
            created.idproduct,
            &ProductChanges::new().price(Money::from_cents(-199).unwrap()),
        )
        .await
        .unwrap();
    assert_eq!(changed.price, Money::from_cents(-199).unwrap());

    let tagged = client
        .products()
        .add_tag(created.idproduct, tag)
        .await
        .unwrap();
    assert_eq!(
        client.products().tags(created.idproduct).await.unwrap(),
        vec![tagged]
    );
    client
        .products()
        .remove_tag(created.idproduct, tag)
        .await
        .unwrap();

    let warehouse = client.warehouses().list_all().await.unwrap()[0].idwarehouse;
    let reason = StockReason::new("picqer-rs live test").unwrap();
    let history_query = StockHistoryQuery::new().idproduct(created.idproduct);
    let history_before = client
        .stock_history()
        .list_all(&history_query)
        .await
        .unwrap()
        .len();
    let added = client
        .products()
        .update_stock(
            created.idproduct,
            warehouse,
            &StockUpdate::adjust(
                StockTarget::NoSpecificLocation,
                NonZeroI64::new(3).unwrap(),
                reason.clone(),
            ),
        )
        .await
        .unwrap();
    assert_eq!(added.level.stock, 3);
    let read = client
        .products()
        .stock_in_warehouse(created.idproduct, warehouse)
        .await
        .unwrap();
    assert_eq!(read, added);
    let reset = client
        .products()
        .update_stock(created.idproduct, warehouse, &StockUpdate::set(0, reason))
        .await
        .unwrap();
    assert_eq!(reset.level.stock, 0);
    let history = client
        .stock_history()
        .list_all(&history_query)
        .await
        .unwrap();
    assert_eq!(history.len(), history_before + 2);
    assert!(
        history[..2]
            .iter()
            .all(|entry| { entry.change_type == Some(Reported::Known(ChangeType::Api)) })
    );

    let settings = client
        .products()
        .update_warehouse_settings(
            created.idproduct,
            warehouse,
            &WarehouseSettingsChanges::new().stock_level_order(5),
        )
        .await
        .unwrap();
    assert_eq!(settings.stock_level_order, Some(5));
    let reset = WarehouseSettingsChanges::new()
        .stock_level_order(0)
        .picking_stock_replenish_trigger(None);
    client
        .products()
        .update_warehouse_settings(created.idproduct, warehouse, &reset)
        .await
        .unwrap();

    let image = NewProductImage::new(PNG).unwrap();
    let added = client
        .products()
        .add_image(created.idproduct, &image)
        .await
        .unwrap();
    assert_eq!(added.contenttype, Reported::Known(ImageType::Png));
    client
        .products()
        .remove_image(created.idproduct, added.idproduct_image)
        .await
        .unwrap();

    if let Some(field) = client
        .product_fields()
        .list_all()
        .await
        .unwrap()
        .into_iter()
        .find(|field| field.field_type == Some(Reported::Known(FieldType::Multicheckbox)))
    {
        let listed: Vec<String> = field.values.clone().unwrap_or_default();
        let value = field.choices(listed.iter().take(2)).unwrap();
        let changed = client
            .products()
            .update(
                created.idproduct,
                &ProductChanges::new().productfields(vec![value]),
            )
            .await
            .unwrap();
        let entry = changed
            .productfields
            .iter()
            .find(|entry| entry.idproductfield == field.idproductfield)
            .unwrap();
        assert_eq!(entry.choices(), listed.iter().take(2).collect::<Vec<_>>());
        client
            .products()
            .update(
                created.idproduct,
                &ProductChanges::new().productfields(vec![field.cleared()]),
            )
            .await
            .unwrap();
    }

    let body = CommentBody::new("picqer-rs live test").unwrap();
    let target = CommentTarget::Product(created.idproduct);
    let comment = client
        .comments()
        .add(target, &NewComment::new(body).show_at_related(false))
        .await
        .unwrap();
    let result = async {
        assert_eq!(comment.show_at_related, Some(false));
        assert!(matches!(
            &comment.source,
            CommentSource::Product { idproduct, .. } if *idproduct == created.idproduct
        ));
        assert!(
            client
                .comments()
                .for_target_all(target)
                .await?
                .contains(&comment)
        );
        assert_eq!(client.comments().get(comment.idcomment).await?, comment);
        let author = match &comment.author {
            Person::User(user) => Commenter::User(user.iduser),
            other => panic!("the API key belongs to a user, not {other:?}"),
        };
        let by_author = client
            .comments()
            .list_all(&CommentQuery::new().author(author))
            .await?;
        assert!(by_author.iter().any(|c| c.idcomment == comment.idcomment));
        Ok::<(), picqer_rs::Error>(())
    }
    .await;
    client.comments().delete(comment.idcomment).await.unwrap();
    result.unwrap();

    let prices = client
        .products()
        .price_history(created.idproduct)
        .await
        .unwrap();
    assert_eq!(
        prices.last().map(|change| change.newprice),
        Some(Money::from_cents(-199).unwrap())
    );

    let inactive = client
        .products()
        .inactivate(created.idproduct)
        .await
        .unwrap();
    assert_eq!(inactive.active, Some(false));
    let query = ProductQuery::new()
        .status(ProductStatus::Inactive)
        .productcode(ProductCode::new(code).unwrap());
    assert_eq!(client.products().list_all(&query).await.unwrap().len(), 1);
}

#[tokio::test]
#[ignore = "needs a Picqer account"]
async fn a_customer_can_be_created_changed_and_deleted() {
    use picqer_rs::customers::{
        AddressChanges, AddressName, CustomerChanges, CustomerName, CustomerNumber, CustomerQuery,
        DiscountPercentage, Language, NewCustomer, NewCustomerAddress,
    };

    let client = client();
    let number = CustomerNumber::new(unique("PRS")).unwrap();
    let tag = client.tags().list_all(&TagQuery::new()).await.unwrap()[0].idtag;
    let new = NewCustomer::new(CustomerName::new("picqer-rs live test").unwrap())
        .customerid(number.clone())
        .language(Language::En)
        .discount(DiscountPercentage::new(12.5).unwrap())
        .address(
            NewCustomerAddress::new(AddressName::new("Home").unwrap(), CountryCode::NL)
                .city("Amsterdam".to_owned())
                .defaultinvoice(true),
        );
    let created = client.customers().create(&new).await.unwrap();

    let result = async {
        assert_eq!(created.customerid.as_deref(), Some(number.as_str()));
        assert_eq!(created.language, Some(Reported::Known(Language::En)));
        assert_eq!(created.discount.map(|d| d.get()), Some(12.5));
        assert_eq!(created.addresses.len(), 1);

        let found = client.customers().find_by_number(&number).await?;
        assert_eq!(found.map(|c| c.idcustomer), Some(created.idcustomer));
        let listed = client
            .customers()
            .list_all(&CustomerQuery::new().customerid(number.clone()))
            .await?;
        assert_eq!(listed.len(), 1);

        let renamed = client
            .customers()
            .update(
                created.idcustomer,
                &CustomerChanges::new()
                    .name(CustomerName::new("picqer-rs live test 2").unwrap())
                    .calculatevat(false),
            )
            .await?;
        assert_eq!(renamed.name, "picqer-rs live test 2");
        assert_eq!(renamed.calculatevat, Some(false));

        let address = client
            .customers()
            .add_address(
                created.idcustomer,
                &NewCustomerAddress::new(AddressName::new("Warehouse").unwrap(), CountryCode::BE),
            )
            .await?;
        let moved = client
            .customers()
            .update_address(
                created.idcustomer,
                address.idcustomer_address,
                &AddressChanges::new().city("Antwerpen".to_owned()),
            )
            .await?;
        assert_eq!(moved.city.as_deref(), Some("Antwerpen"));
        assert_eq!(
            client
                .customers()
                .addresses_all(created.idcustomer)
                .await?
                .len(),
            2
        );
        client
            .customers()
            .remove_address(created.idcustomer, address.idcustomer_address)
            .await?;

        let field = client
            .customer_fields()
            .list_all()
            .await?
            .into_iter()
            .find(|field| {
                field.field_type == Some(Reported::Known(picqer_rs::fields::FieldType::Text))
            })
            .expect("the account has a text customer field");
        let with_value = client
            .customers()
            .update(
                created.idcustomer,
                &CustomerChanges::new().customerfields(vec![field.value("live").unwrap()]),
            )
            .await?;
        let stored = client.customers().get(created.idcustomer).await?;
        assert_eq!(with_value.idcustomer, stored.idcustomer);
        assert_eq!(stored.customerfields.len(), 1);
        assert_eq!(
            stored.customerfields[0].idcustomerfield,
            field.idcustomerfield
        );
        assert_eq!(stored.customerfields[0].value.as_deref(), Some("live"));
        let emptied = client
            .customers()
            .update(
                created.idcustomer,
                &CustomerChanges::new().customerfields(vec![field.cleared()]),
            )
            .await?;
        assert_eq!(emptied.idcustomer, created.idcustomer);
        assert!(
            client
                .customers()
                .get(created.idcustomer)
                .await?
                .customerfields
                .is_empty()
        );

        let choices: Vec<_> = client
            .customer_fields()
            .list_all()
            .await?
            .into_iter()
            .filter(|field| {
                matches!(
                    field.field_type,
                    Some(Reported::Known(
                        picqer_rs::fields::FieldType::Radio | picqer_rs::fields::FieldType::Select
                    ))
                )
            })
            .collect();
        assert!(!choices.is_empty());
        let values = choices
            .iter()
            .map(|field| {
                assert!(field.value("not on the list").is_err());
                field.value("a").unwrap()
            })
            .collect();
        client
            .customers()
            .update(
                created.idcustomer,
                &CustomerChanges::new().customerfields(values),
            )
            .await?;
        let stored = client.customers().get(created.idcustomer).await?;
        assert_eq!(stored.customerfields.len(), choices.len());
        assert!(
            stored
                .customerfields
                .iter()
                .all(|entry| entry.value.as_deref() == Some("a"))
        );
        client
            .customers()
            .update(
                created.idcustomer,
                &CustomerChanges::new()
                    .customerfields(choices.iter().map(|field| field.cleared()).collect()),
            )
            .await?;

        for field in client.customer_fields().list_all().await? {
            let listed = field.values.clone().unwrap_or_default();
            let value = if field.field_type
                == Some(Reported::Known(picqer_rs::fields::FieldType::Multicheckbox))
            {
                field.choices(listed.iter().take(2)).unwrap()
            } else {
                field
                    .value(
                        listed
                            .first()
                            .cloned()
                            .unwrap_or_else(|| "live\ntest".to_owned()),
                    )
                    .unwrap()
            };
            let text = serde_json::to_value(&value).unwrap()["value"]
                .as_str()
                .unwrap()
                .to_owned();
            client
                .customers()
                .update(
                    created.idcustomer,
                    &CustomerChanges::new().customerfields(vec![value]),
                )
                .await?;
            let stored = client.customers().get(created.idcustomer).await?;
            assert_eq!(stored.customerfields.len(), 1);
            assert_eq!(
                stored.customerfields[0].value.as_deref(),
                Some(text.as_str())
            );
            client
                .customers()
                .update(
                    created.idcustomer,
                    &CustomerChanges::new().customerfields(vec![field.cleared()]),
                )
                .await?;
        }
        assert!(
            client
                .customers()
                .get(created.idcustomer)
                .await?
                .customerfields
                .is_empty()
        );

        let tagged = client.customers().add_tag(created.idcustomer, tag).await?;
        assert_eq!(
            client.customers().tags(created.idcustomer).await?,
            vec![tagged]
        );
        client
            .customers()
            .remove_tag(created.idcustomer, tag)
            .await?;
        Ok::<(), picqer_rs::Error>(())
    }
    .await;
    client.customers().delete(created.idcustomer).await.unwrap();
    result.unwrap();
    assert!(
        client
            .customers()
            .get(created.idcustomer)
            .await
            .unwrap_err()
            .is_not_found()
    );
}

async fn wait_for_status(
    client: &Client,
    id: picqer_rs::types::OrderId,
    wanted: &[picqer_rs::orders::OrderStatus],
    seconds: u64,
) -> picqer_rs::orders::Order {
    let mut waited = 0;
    loop {
        let order = client.orders().get(id).await.unwrap();
        let known = match &order.status {
            Reported::Known(status) => Some(*status),
            Reported::Unknown(_) => None,
        };
        if known.is_some_and(|status| wanted.contains(&status)) || waited >= seconds {
            return order;
        }
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        waited += 3;
    }
}

async fn cancel_when_ready(client: &Client, id: picqer_rs::types::OrderId) {
    for _ in 0..10 {
        match client.orders().cancel(id).await {
            Ok(()) => return,
            Err(error)
                if error.as_api().and_then(|api| api.code.as_ref())
                    == Some(&Reported::Known(picqer_rs::ErrorCode::OrderBeingProcessed)) =>
            {
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            }
            Err(error) => panic!("{error}"),
        }
    }
    panic!("the order stayed busy");
}

async fn order_for_the_tests(
    client: &Client,
    reference: &str,
    amount: u32,
) -> picqer_rs::orders::Order {
    use picqer_rs::customers::CustomerQuery;
    use picqer_rs::orders::{NewOrder, NewOrderProduct, OrderQuery, OrderStatus};
    use picqer_rs::types::NonEmpty;

    for status in [OrderStatus::Processing, OrderStatus::Paused] {
        for left_over in client
            .orders()
            .list_all(&OrderQuery::new().reference(reference).status(status))
            .await
            .unwrap()
        {
            client
                .orders()
                .force_cancel(left_over.idorder)
                .await
                .unwrap();
            wait_for_status(client, left_over.idorder, &[OrderStatus::Cancelled], 90).await;
        }
    }
    let all_products = client
        .products()
        .list_all(&ProductQuery::new())
        .await
        .unwrap();
    let enough_stock = |product: &picqer_rs::products::Product| {
        product
            .stock
            .iter()
            .any(|level| level.freestock >= i64::from(amount))
            && product.stock.iter().all(|level| level.freestock >= 0)
    };
    for status in [OrderStatus::Concept, OrderStatus::Cancelled] {
        let found = client
            .orders()
            .list_all(&OrderQuery::new().reference(reference).status(status))
            .await
            .unwrap();
        let suitable = found.into_iter().find(|order| {
            order.products.iter().all(|line| {
                all_products
                    .iter()
                    .find(|product| product.idproduct == line.idproduct)
                    .is_some_and(enough_stock)
            })
        });
        if let Some(order) = suitable {
            if status == OrderStatus::Cancelled {
                client
                    .orders()
                    .undo_cancellation(order.idorder)
                    .await
                    .unwrap();
            }
            return wait_for_status(client, order.idorder, &[OrderStatus::Concept], 60).await;
        }
    }
    let customer = client
        .customers()
        .list_all(&CustomerQuery::new())
        .await
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    let product = all_products
        .iter()
        .find(|product| {
            product.idfulfilment_customer == customer.idfulfilment_customer && enough_stock(product)
        })
        .expect("a product of the customer with free stock");
    let line = NewOrderProduct::by_id(
        product.idproduct,
        std::num::NonZeroU32::new(amount).unwrap(),
    );
    let order = NewOrder::for_customer(customer.idcustomer, NonEmpty::new(vec![line]).unwrap())
        .reference(reference);
    client.orders().create(&order).await.unwrap()
}

#[tokio::test]
#[ignore = "needs a Picqer account"]
async fn an_order_can_be_changed_and_cancelled() {
    use picqer_rs::customers::DiscountPercentage;
    use picqer_rs::orders::{
        NewOrderLine, OrderAddressChanges, OrderChanges, OrderLineChanges, OrderStatus,
    };
    use std::num::NonZeroU32;

    let client = client();
    let order = order_for_the_tests(&client, "picqer-rs-live", 1).await;
    assert_eq!(order.status, Reported::Known(OrderStatus::Concept));
    let id = order.idorder;
    let tag = client.tags().list_all(&TagQuery::new()).await.unwrap()[0].idtag;

    let result = async {
        let changed = client
            .orders()
            .update(
                id,
                &OrderChanges::new()
                    .telephone("+31 6 12345678")
                    .discount(DiscountPercentage::new(15.0).unwrap())
                    .delivery(OrderAddressChanges::new().city("Utrecht")),
            )
            .await?;
        assert_eq!(changed.telephone.as_deref(), Some("+31 6 12345678"));
        assert_eq!(changed.discount.map(|d| d.get()), Some(15.0));
        assert_eq!(changed.deliverycity.as_deref(), Some("Utrecht"));
        assert_eq!(changed.reference.as_deref(), Some("picqer-rs-live"));

        let product = order.products[0].idproduct;
        let line = client
            .orders()
            .add_line(
                id,
                &NewOrderLine::new(product, NonZeroU32::new(1).unwrap()).remarks("live test"),
            )
            .await?;
        let lines = client.orders().lines(id).await?;
        assert!(
            lines
                .iter()
                .any(|l| l.idorder_product == line.idorder_product)
        );
        let updated = client
            .orders()
            .update_line(
                id,
                line.idorder_product,
                &OrderLineChanges::new().price(Money::from_cents(1234).unwrap()),
            )
            .await?;
        assert_eq!(updated.price, Some(Money::from_cents(1234).unwrap()));

        let tagged = client.orders().add_tag(id, tag).await?;
        assert_eq!(
            client
                .orders()
                .tags(id, picqer_rs::pagination::Page::FIRST)
                .await?,
            vec![tagged]
        );
        client.orders().remove_tag(id, tag).await?;

        let field = client
            .order_fields()
            .list_all()
            .await?
            .into_iter()
            .find(|field| {
                field.field_type == Some(Reported::Known(picqer_rs::fields::FieldType::Text))
            })
            .expect("the account has a text order field");
        let value = field.value("12").unwrap();
        let entry = client.orders().set_field(id, &value).await?;
        assert_eq!(entry.idorderfield, field.idorderfield);
        assert_eq!(entry.value.as_deref(), Some("12"));
        let entries = client.orders().fields(id).await?;
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].value.as_deref(), Some("12"));
        assert_eq!(client.orders().get(id).await?.orderfields, entries);
        client.orders().remove_field(id, field.idorderfield).await?;
        assert!(client.orders().fields(id).await?.is_empty());

        for field in client
            .order_fields()
            .list_all()
            .await?
            .into_iter()
            .filter(|field| {
                matches!(
                    field.field_type,
                    Some(Reported::Known(
                        picqer_rs::fields::FieldType::Radio | picqer_rs::fields::FieldType::Select
                    ))
                )
            })
        {
            assert!(field.value("not on the list").is_err());
            let listed = field.value("a").unwrap();
            let entry = client.orders().set_field(id, &listed).await?;
            assert_eq!(entry.value.as_deref(), Some("a"));
            client.orders().remove_field(id, field.idorderfield).await?;
        }
        assert!(client.orders().fields(id).await?.is_empty());

        for field in client.order_fields().list_all().await? {
            let listed = field.values.clone().unwrap_or_default();
            let value = if field.field_type
                == Some(Reported::Known(picqer_rs::fields::FieldType::Multicheckbox))
            {
                field.choices(listed.iter().take(2)).unwrap()
            } else {
                field
                    .value(
                        listed
                            .first()
                            .cloned()
                            .unwrap_or_else(|| "live\ntest".to_owned()),
                    )
                    .unwrap()
            };
            let text = serde_json::to_value(&value).unwrap()["value"]
                .as_str()
                .unwrap()
                .to_owned();
            let entry = client.orders().set_field(id, &value).await?;
            assert_eq!(entry.value.as_deref(), Some(text.as_str()));
            let stored = client.orders().fields(id).await?;
            assert_eq!(stored.len(), 1);
            assert_eq!(stored[0].value.as_deref(), Some(text.as_str()));
            client.orders().remove_field(id, field.idorderfield).await?;
        }
        assert!(client.orders().fields(id).await?.is_empty());

        client.orders().allocate(id).await?;
        client.orders().deallocate(id).await?;
        let status = client.orders().product_status(id).await?;
        assert!(status.iter().any(|line| line.idproduct == product));
        Ok::<(), picqer_rs::Error>(())
    }
    .await;

    let lines = client.orders().lines(id).await.unwrap();
    for line in lines.iter().skip(1) {
        client
            .orders()
            .remove_line(id, line.idorder_product)
            .await
            .unwrap();
    }
    cancel_when_ready(&client, id).await;
    let cancelled = wait_for_status(&client, id, &[OrderStatus::Cancelled], 60).await;
    result.unwrap();
    assert_eq!(cancelled.status, Reported::Known(OrderStatus::Cancelled));
}

#[tokio::test]
#[ignore = "needs a Picqer account, takes a minute"]
async fn an_order_can_be_processed_paused_resumed_and_cancelled_by_force() {
    use picqer_rs::orders::OrderStatus;
    use picqer_rs::types::PauseReason;

    let client = client();
    let order = order_for_the_tests(&client, "picqer-rs-live-process", 1).await;
    let id = order.idorder;

    let result = async {
        client.orders().process(id).await?;
        let processing = wait_for_status(&client, id, &[OrderStatus::Processing], 150).await;
        assert_eq!(processing.status, Reported::Known(OrderStatus::Processing));

        let reason = PauseReason::new("picqer-rs live test").unwrap();
        client.orders().pause(id, &reason).await?;
        let paused = wait_for_status(&client, id, &[OrderStatus::Paused], 60).await;
        assert_eq!(paused.paused_reason.as_deref(), Some("picqer-rs live test"));
        client.orders().resume(id).await?;
        let resumed = wait_for_status(&client, id, &[OrderStatus::Processing], 60).await;
        assert_eq!(resumed.paused_reason, None);

        assert!(client.orders().cancel(id).await.is_err());
        Ok::<(), picqer_rs::Error>(())
    }
    .await;

    client.orders().force_cancel(id).await.unwrap();
    let cancelled = wait_for_status(&client, id, &[OrderStatus::Cancelled], 90).await;
    result.unwrap();
    assert_eq!(cancelled.status, Reported::Known(OrderStatus::Cancelled));
}

#[tokio::test]
#[ignore = "needs a Picqer account, takes a minute"]
async fn a_picklist_can_be_added_to_a_batch_and_taken_out_again() {
    use picqer_rs::orders::OrderStatus;
    use picqer_rs::picklist_batch_presets::PresetStatus;
    use picqer_rs::picklist_batches::{BatchPdf, BatchQuery, BatchStatus, BatchType};
    use picqer_rs::picklists::PicklistStatus;

    let client = client();
    let batches = client.picklist_batches();
    let open = batches
        .list_all(&BatchQuery::new().status(BatchStatus::Open))
        .await
        .unwrap();
    let singles = open
        .iter()
        .find(|batch| batch.batch_type == Some(Reported::Known(BatchType::Singles)))
        .expect("the account has an open singles batch");
    let batch_id = singles.idpicklist_batch;
    let before = batches.get(batch_id).await.unwrap();
    assert_eq!(Some(before.picklists.len() as i64), before.total_picklists);
    assert!(!before.products.is_empty());
    assert_eq!(
        batches
            .list_all(&BatchQuery::new().batch_type(BatchType::Singles))
            .await
            .unwrap()
            .iter()
            .filter(|batch| batch.status == Some(Reported::Known(BatchStatus::Open)))
            .count(),
        open.iter()
            .filter(|batch| batch.batch_type == Some(Reported::Known(BatchType::Singles)))
            .count()
    );
    assert!(
        batches
            .list_all(&BatchQuery::new().status(BatchStatus::Completed))
            .await
            .unwrap()
            .iter()
            .all(|batch| batch.status == Some(Reported::Known(BatchStatus::Completed)))
    );

    let presets = client.picklist_batch_presets();
    for preset in presets.list_all(PresetStatus::Active).await.unwrap() {
        assert_eq!(preset.active, Some(true));
        let single = presets.get(preset.idpicklist_batch_preset).await.unwrap();
        assert!(single.picklists_count.is_some());
    }
    assert!(
        presets
            .list_all(PresetStatus::Inactive)
            .await
            .unwrap()
            .iter()
            .all(|preset| preset.active == Some(false))
    );

    let product = before.products[0].idproduct;
    let in_batch = batches
        .picklists_of_product(batch_id, product)
        .await
        .unwrap();
    assert!(!in_batch.is_empty());
    assert!(
        in_batch
            .iter()
            .all(|picklist| picklist.idpicklist_batch == Some(batch_id))
    );
    for content in [
        BatchPdf::Batch,
        BatchPdf::WithPicklists,
        BatchPdf::WithPackinglists,
    ] {
        assert!(
            batches
                .pdf(batch_id, content)
                .await
                .unwrap()
                .starts_with(b"%PDF")
        );
    }
    if let Some(user) = &before.assigned_to {
        let assigned = batches.assign(batch_id, Some(user.iduser)).await.unwrap();
        assert_eq!(assigned.assigned_to, before.assigned_to);
    }

    let order = order_for_the_tests(&client, "picqer-rs-live-batch", 1).await;
    let id = order.idorder;
    let result = async {
        client.orders().process(id).await?;
        wait_for_status(&client, id, &[OrderStatus::Processing], 150).await;
        let picklist = client
            .orders()
            .get(id)
            .await?
            .picklists
            .iter()
            .find(|picklist| picklist.status == Reported::Known(PicklistStatus::New))
            .expect("processing made a picklist")
            .idpicklist;

        let added = batches.add_picklist(batch_id, picklist).await?;
        assert_eq!(
            added.total_picklists,
            Some(before.picklists.len() as i64 + 1)
        );
        assert!(
            added
                .picklists
                .iter()
                .any(|entry| entry.idpicklist == picklist)
        );
        assert_eq!(
            client.picklists().get(picklist).await?.idpicklist_batch,
            Some(batch_id)
        );
        let removed = batches.remove_picklist(batch_id, picklist).await?;
        assert_eq!(removed.total_picklists, before.total_picklists);
        assert!(
            removed
                .picklists
                .iter()
                .all(|entry| entry.idpicklist != picklist)
        );
        assert_eq!(
            client.picklists().get(picklist).await?.idpicklist_batch,
            None
        );
        let again = batches
            .remove_picklist(batch_id, picklist)
            .await
            .unwrap_err();
        assert!(again.is_not_found());
        Ok::<(), picqer_rs::Error>(())
    }
    .await;

    client.orders().force_cancel(id).await.unwrap();
    wait_for_status(&client, id, &[OrderStatus::Cancelled], 90).await;
    result.unwrap();
    let after = batches.get(batch_id).await.unwrap();
    assert_eq!(after.total_picklists, before.total_picklists);
    assert_eq!(after.status, Some(Reported::Known(BatchStatus::Open)));
}

/// Makes batches, which Picqer cannot delete: each one stays behind as a completed batch. Only runs
/// with `PICQER_LIVE_CREATE_BATCHES=1`.
#[tokio::test]
#[ignore = "needs a Picqer account, leaves completed batches behind"]
async fn a_batch_can_be_created_from_picklists_and_from_a_preset() {
    use picqer_rs::orders::OrderStatus;
    use picqer_rs::picklist_batch_presets::PresetStatus;
    use picqer_rs::picklist_batches::{BatchStatus, NewBatch};
    use picqer_rs::picklists::PicklistStatus;
    use picqer_rs::types::NonEmpty;
    use std::num::NonZeroU32;

    load_env();
    if std::env::var("PICQER_LIVE_CREATE_BATCHES").as_deref() != Ok("1") {
        eprintln!("skipped: set PICQER_LIVE_CREATE_BATCHES=1 to make batches");
        return;
    }
    let client = client();
    let batches = client.picklist_batches();
    let order = order_for_the_tests(&client, "picqer-rs-live-batch-create", 1).await;
    let id = order.idorder;
    let result = async {
        client.orders().process(id).await?;
        wait_for_status(&client, id, &[OrderStatus::Processing], 150).await;
        let picklist = client
            .orders()
            .get(id)
            .await?
            .picklists
            .iter()
            .find(|picklist| picklist.status == Reported::Known(PicklistStatus::New))
            .expect("processing made a picklist")
            .idpicklist;

        let made = batches
            .create(&NewBatch::from_picklists(
                NonEmpty::new(vec![picklist]).unwrap(),
            ))
            .await?;
        assert_eq!(made.status, Some(Reported::Known(BatchStatus::Open)));
        assert_eq!(made.total_picklists, Some(1));
        assert_eq!(made.picklists[0].idpicklist, picklist);
        assert_eq!(made.products.len(), 1);
        batches
            .remove_picklist(made.idpicklist_batch, picklist)
            .await?;

        let preset = client
            .picklist_batch_presets()
            .list_all(PresetStatus::Active)
            .await?
            .into_iter()
            .next()
            .expect("the account has a preset");
        let from_preset = batches
            .create(
                &NewBatch::from_preset(preset.idpicklist_batch_preset)
                    .preferred_number_of_picklists(NonZeroU32::new(1).unwrap()),
            )
            .await?;
        assert_eq!(
            from_preset.idpicklist_batch_preset,
            Some(preset.idpicklist_batch_preset)
        );
        assert_eq!(from_preset.total_picklists, Some(1));
        for entry in &from_preset.picklists {
            batches
                .remove_picklist(from_preset.idpicklist_batch, entry.idpicklist)
                .await?;
        }
        Ok::<(), picqer_rs::Error>(())
    }
    .await;

    client.orders().force_cancel(id).await.unwrap();
    wait_for_status(&client, id, &[OrderStatus::Cancelled], 90).await;
    result.unwrap();
}

#[tokio::test]
#[ignore = "needs a Picqer account, takes a minute"]
async fn a_shipment_can_be_made_listed_and_read() {
    use picqer_rs::orders::OrderStatus;
    use picqer_rs::shipments::{
        CarrierKey, DocumentFormat, DocumentType, NewShipment, ShipmentQuery,
    };
    use picqer_rs::types::Grams;

    let client = client();
    let order = order_for_the_tests(&client, "picqer-rs-live-shipment", 1).await;
    let id = order.idorder;

    let result = async {
        client.orders().process(id).await?;
        wait_for_status(&client, id, &[OrderStatus::Processing], 150).await;
        let picklist = client
            .orders()
            .get(id)
            .await?
            .picklists
            .iter()
            .find(|picklist| {
                picklist.status == Reported::Known(picqer_rs::picklists::PicklistStatus::New)
            })
            .expect("processing made a picklist")
            .idpicklist;

        let provider = client
            .shipping_providers()
            .list_all()
            .await?
            .into_iter()
            .find(|provider| provider.provider.as_deref() == Some("Manual"))
            .expect("the account has a Manual shipping provider");
        let profile = provider.profiles[0].idshippingprovider_profile;
        let methods = client.shipments().methods(picklist).await?;
        let method = methods
            .iter()
            .find(|method| method.idshippingprovider_profile == profile)
            .expect("the Manual profile is allowed for the picklist");
        assert_eq!(method.multi_colli_supported, Some(false));
        assert!(method.extra_fields.iter().any(|field| field.key == "code"));

        let packaging = client
            .packagings()
            .list_all(picqer_rs::packagings::PackagingStatus::Active)
            .await?
            .into_iter()
            .find(|packaging| !packaging.name.starts_with("picqer-rs-live"))
            .expect("the account has a packaging of its own");
        let code = unique("TRK");
        let url = format!("https://example.com/t/{code}");
        let made = client
            .shipments()
            .create(
                picklist,
                &NewShipment::new(profile)
                    .idpackaging(packaging.idpackaging)
                    .weight(Grams::new(900))
                    .extrafield("code", code.clone())
                    .extrafield("tracking_url", url.clone()),
            )
            .await?;
        assert!(!made.cancelled);
        assert_eq!(made.idpicklist, Some(picklist));
        assert_eq!(made.idorder, Some(id));
        assert_eq!(
            made.idcompany_shippingprovider,
            Some(provider.idshippingprovider)
        );
        assert_eq!(made.idcompany_shippingprovider_profile, Some(profile));
        assert_eq!(made.provider.as_deref(), Some("Manual"));
        assert_eq!(made.carrier_key, Some(Reported::Known(CarrierKey::Postal)));
        assert_eq!(made.weight, Some(Grams::new(900)));
        assert_eq!(made.idpackaging, Some(packaging.idpackaging));
        assert_eq!(made.parcels.len(), 1);
        let parcel = &made.parcels[0];
        assert_eq!(parcel.tracking_code.as_deref(), Some(code.as_str()));
        assert_eq!(parcel.tracking_url.as_deref(), Some(url.as_str()));
        assert_eq!(parcel.weight, Some(Grams::new(900)));
        assert_eq!(parcel.documents.len(), 1);
        assert_eq!(
            parcel.documents[0].document_type,
            Some(Reported::Known(DocumentType::Label))
        );
        assert_eq!(
            parcel.documents[0].format,
            Some(Reported::Known(DocumentFormat::Pdf))
        );

        assert_eq!(client.shipments().get(made.idshipment).await?, made);
        let on_the_picklist = client.shipments().for_picklist(picklist).await?;
        assert_eq!(on_the_picklist, vec![made.clone()]);
        let mut found = Vec::new();
        for _ in 0..20 {
            found = client
                .shipments()
                .list_all(&ShipmentQuery::new().search(code.clone()))
                .await?;
            if !found.is_empty() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
        assert_eq!(found, vec![made.clone()]);

        let failed = client
            .shipments()
            .create(
                picklist,
                &NewShipment::new(profile).parcels(
                    picqer_rs::types::NonEmpty::new(vec![
                        picqer_rs::shipments::NewParcel::new(
                            packaging.idpackaging,
                            Grams::new(500)
                        );
                        2
                    ])
                    .unwrap(),
                ),
            )
            .await
            .unwrap_err();
        assert_eq!(
            failed.as_api().unwrap().code,
            Some(Reported::Known(picqer_rs::ErrorCode::CannotCreateShipment))
        );
        Ok::<(), picqer_rs::Error>(())
    }
    .await;

    client.orders().force_cancel(id).await.unwrap();
    wait_for_status(&client, id, &[OrderStatus::Cancelled], 90).await;
    result.unwrap();
}

#[tokio::test]
#[ignore = "needs a Picqer account, takes a minute"]
async fn a_picklist_can_be_changed_picked_split_and_cancelled() {
    use picqer_rs::orders::OrderStatus;
    use picqer_rs::picklists::{
        Pick, PickTarget, PicklistChanges, PicklistStatus, Split, SplitLine,
    };
    use picqer_rs::types::{NonEmpty, PauseReason};
    use std::num::NonZeroU32;

    let client = client();
    let order = order_for_the_tests(&client, "picqer-rs-live-picklist", 3).await;
    let id = order.idorder;
    let one = NonZeroU32::new(1).unwrap();

    let result = async {
        let user = match client
            .comments()
            .list_all(&CommentQuery::new())
            .await?
            .first()
        {
            Some(comment) => match &comment.author {
                Person::User(user) => Some(user.iduser),
                _ => None,
            },
            None => None,
        };
        client.orders().process(id).await?;
        wait_for_status(&client, id, &[OrderStatus::Processing], 150).await;
        let embedded = client.orders().get(id).await?.picklists;
        let picklist_id = embedded
            .iter()
            .find(|picklist| picklist.status == Reported::Known(PicklistStatus::New))
            .expect("processing made a picklist")
            .idpicklist;
        let picklists = client.picklists();

        let picklist = picklists.get(picklist_id).await?;
        assert_eq!(picklist.idorder, Some(id));
        assert_eq!(picklist.totalproducts, 3);
        let line = picklist.products[0].clone();
        let code = ProductCode::new(line.productcode.clone()).unwrap();

        let urgent = picklists
            .update(picklist_id, &PicklistChanges::new().urgent(true))
            .await?;
        assert!(urgent.urgent);
        picklists
            .update(picklist_id, &PicklistChanges::new().urgent(false))
            .await?;

        if let Some(user) = user {
            let assigned = picklists.assign(picklist_id, user).await?;
            assert_eq!(assigned.assigned_to_iduser, Some(user));
            picklists.unassign(picklist_id).await?;
        }

        let until: picqer_rs::types::DateTime = "2035-01-01 12:00:00".parse().unwrap();
        let snoozed = picklists.snooze(picklist_id, until).await?;
        assert_eq!(snoozed.status, Reported::Known(PicklistStatus::Snoozed));
        let back = picklists.unsnooze(picklist_id).await?;
        assert_eq!(back.status, Reported::Known(PicklistStatus::New));

        let reason = PauseReason::new("picqer-rs live test").unwrap();
        let paused = picklists.pause(picklist_id, &reason).await?;
        assert_eq!(paused.status, Reported::Known(PicklistStatus::Paused));
        let resumed = picklists.resume(picklist_id).await?;
        assert_eq!(resumed.status, Reported::Known(PicklistStatus::New));

        let picked = picklists
            .pick(picklist_id, &Pick::new(PickTarget::Product(code), one))
            .await?;
        assert_eq!(picked.totalpicked, 1);
        let undone = picklists
            .pick(
                picklist_id,
                &Pick::undo(PickTarget::Line(line.idpicklist_product), one),
            )
            .await?;
        assert_eq!(undone.totalpicked, 0);

        let advice = picklists.packaging_advice(picklist_id).await?;
        assert_eq!(advice.idpackaging, None);
        let packaging = client
            .packagings()
            .list_all(picqer_rs::packagings::PackagingStatus::Active)
            .await?
            .into_iter()
            .find(|packaging| !packaging.name.starts_with("picqer-rs-live"))
            .expect("the account has a packaging of its own");
        let advised = picklists
            .set_packaging_advice(picklist_id, packaging.idpackaging)
            .await?;
        assert_eq!(advised.idpackaging, Some(packaging.idpackaging));
        assert_eq!(
            advised.source,
            Some(Reported::Known(picqer_rs::picklists::AdviceSource::Api))
        );
        assert_eq!(picklists.packaging_advice(picklist_id).await?, advised);
        assert!(
            picklists
                .picklist_pdf(picklist_id)
                .await?
                .starts_with(b"%PDF")
        );
        assert!(
            picklists
                .packinglist_pdf(picklist_id)
                .await?
                .starts_with(b"%PDF")
        );

        let lines = NonEmpty::new(vec![SplitLine::new(line.idpicklist_product, one)]).unwrap();
        let new = picklists.split(picklist_id, &Split::new(lines)).await?;
        assert_eq!(new.idorder, Some(id));
        assert_eq!(new.totalproducts, 1);
        assert_eq!(picklists.get(picklist_id).await?.totalproducts, 2);
        picklists.cancel(new.idpicklist).await?;
        Ok::<(), picqer_rs::Error>(())
    }
    .await;

    client.orders().force_cancel(id).await.unwrap();
    let cancelled = wait_for_status(&client, id, &[OrderStatus::Cancelled], 90).await;
    result.unwrap();
    assert_eq!(cancelled.status, Reported::Known(OrderStatus::Cancelled));
}

#[tokio::test]
#[ignore = "needs a Picqer account"]
async fn a_supplier_can_be_created_and_changed() {
    use picqer_rs::customers::Language;
    use picqer_rs::suppliers::{NewSupplier, SupplierChanges, SupplierName};

    let client = client();
    let name = "picqer-rs-live-supplier";
    let existing = client
        .suppliers()
        .list_all()
        .await
        .unwrap()
        .into_iter()
        .find(|supplier| supplier.name == name);
    let supplier = match existing {
        Some(supplier) => supplier,
        None => client
            .suppliers()
            .create(&NewSupplier::new(SupplierName::new(name).unwrap()))
            .await
            .unwrap(),
    };

    let changed = client
        .suppliers()
        .update(
            supplier.idsupplier,
            &SupplierChanges::new()
                .country(CountryCode::BE)
                .language(Language::En)
                .city("Antwerpen".to_owned())
                .remarks("live test".to_owned()),
        )
        .await
        .unwrap();
    assert_eq!(changed.name, name);
    assert_eq!(changed.country, Some(Reported::Known(CountryCode::BE)));
    assert_eq!(changed.language, Some(Reported::Known(Language::En)));
    assert_eq!(changed.city.as_deref(), Some("Antwerpen"));
    let back = client
        .suppliers()
        .update(
            supplier.idsupplier,
            &SupplierChanges::new()
                .country(CountryCode::NL)
                .language(Language::Nl)
                .city("Utrecht".to_owned()),
        )
        .await
        .unwrap();
    assert_eq!(back.language, Some(Reported::Known(Language::Nl)));
    assert_eq!(
        client
            .suppliers()
            .get(supplier.idsupplier)
            .await
            .unwrap()
            .city,
        back.city
    );
}

#[tokio::test]
#[ignore = "needs a Picqer account"]
async fn a_purchase_order_can_be_changed_and_its_lines_too() {
    use picqer_rs::purchase_orders::{
        NewPurchaseOrder, NewPurchaseOrderProduct, PurchaseOrderChanges, PurchaseOrderLineChanges,
        PurchaseOrderQuery, PurchaseOrderStatus,
    };
    use std::num::NonZeroU32;

    let client = client();
    let one = NonZeroU32::new(1).unwrap();
    let products = client
        .products()
        .list_all(&ProductQuery::new())
        .await
        .unwrap();
    let first = products
        .iter()
        .find(|product| product.idfulfilment_customer.is_some())
        .unwrap();
    let customer = first.idfulfilment_customer.unwrap();
    let second = products
        .iter()
        .find(|product| {
            product.idfulfilment_customer == Some(customer) && product.idproduct != first.idproduct
        })
        .unwrap();
    let supplier = client.suppliers().list_all().await.unwrap()[0].idsupplier;

    let reference = "picqer-rs-live";
    let existing = client
        .purchase_orders()
        .list_all(&PurchaseOrderQuery::new().status(PurchaseOrderStatus::Concept))
        .await
        .unwrap()
        .into_iter()
        .find(|order| order.supplier_orderid.as_deref() == Some(reference));
    let order = match existing {
        Some(order) => order,
        None => client
            .purchase_orders()
            .create(
                &NewPurchaseOrder::for_supplier(supplier)
                    .idfulfilment_customer(customer)
                    .supplier_orderid(reference)
                    .product(NewPurchaseOrderProduct::new(first.idproduct, one)),
            )
            .await
            .unwrap(),
    };
    let id = order.idpurchaseorder;
    assert_eq!(order.status, Reported::Known(PurchaseOrderStatus::Concept));

    let changed = client
        .purchase_orders()
        .update(
            id,
            &PurchaseOrderChanges::new()
                .remarks("live test")
                .delivery_date("2031-01-02".parse().unwrap()),
        )
        .await
        .unwrap();
    assert_eq!(changed.remarks.as_deref(), Some("live test"));
    assert_eq!(
        changed
            .delivery_date
            .map(|date| date.to_string())
            .as_deref(),
        Some("2031-01-02")
    );

    let lines = client.purchase_orders().lines(id).await.unwrap();
    for line in lines
        .iter()
        .filter(|line| line.idproduct == second.idproduct)
    {
        client
            .purchase_orders()
            .remove_line(id, line.idpurchaseorder_product)
            .await
            .unwrap();
    }
    let added = client
        .purchase_orders()
        .add_line(
            id,
            &NewPurchaseOrderProduct::new(second.idproduct, NonZeroU32::new(4).unwrap())
                .price(Money::from_cents(250).unwrap()),
        )
        .await
        .unwrap();
    assert_eq!(added.amount, 4);
    assert_eq!(added.price, Some(Money::from_cents(250).unwrap()));
    let updated = client
        .purchase_orders()
        .update_line(
            id,
            added.idpurchaseorder_product,
            &PurchaseOrderLineChanges::new()
                .amount(NonZeroU32::new(7).unwrap())
                .price(Money::from_cents(325).unwrap()),
        )
        .await
        .unwrap();
    assert_eq!(
        (updated.amount, updated.price),
        (7, Some(Money::from_cents(325).unwrap()))
    );
    client
        .purchase_orders()
        .remove_line(id, added.idpurchaseorder_product)
        .await
        .unwrap();
    let remaining = client.purchase_orders().lines(id).await.unwrap();
    assert!(
        remaining
            .iter()
            .all(|line| line.idproduct != second.idproduct)
    );
}

#[tokio::test]
#[ignore = "needs a Picqer account"]
async fn a_packaging_can_be_created_changed_and_deactivated() {
    use picqer_rs::packagings::{NewPackaging, PackagingChanges, PackagingName, PackagingStatus};

    let client = client();
    let name = "picqer-rs-live-packaging";
    let find = |status| {
        let client = &client;
        async move {
            client
                .packagings()
                .list_all(status)
                .await
                .unwrap()
                .into_iter()
                .find(|packaging| packaging.name == name)
        }
    };
    let existing = match find(PackagingStatus::Active).await {
        Some(packaging) => Some(packaging),
        None => find(PackagingStatus::Inactive).await,
    };
    let packaging = match existing {
        Some(packaging) => packaging,
        None => client
            .packagings()
            .create(&NewPackaging::new(PackagingName::new(name).unwrap()))
            .await
            .unwrap(),
    };
    let id = packaging.idpackaging;

    let changed = client
        .packagings()
        .update(
            id,
            &PackagingChanges::new()
                .dimensions(10, 20, 30)
                .barcode("picqer-rs-live")
                .use_in_auto_advice(false)
                .active(true),
        )
        .await
        .unwrap();
    assert_eq!(
        (changed.length, changed.width, changed.height),
        (Some(10), Some(20), Some(30))
    );
    assert_eq!(changed.barcode.as_deref(), Some("picqer-rs-live"));
    assert_eq!(changed.use_in_auto_advice, Some(false));
    let cleared = client
        .packagings()
        .update(
            id,
            &PackagingChanges::new()
                .without_barcode()
                .use_in_auto_advice(true),
        )
        .await
        .unwrap();
    assert_eq!(cleared.barcode, None);
    let inactive = client
        .packagings()
        .update(id, &PackagingChanges::new().active(false))
        .await
        .unwrap();
    assert!(!inactive.active);
    assert!(find(PackagingStatus::Inactive).await.is_some());
    client
        .packagings()
        .update(id, &PackagingChanges::new().active(true))
        .await
        .unwrap();
}

#[tokio::test]
#[ignore = "needs a Picqer account"]
async fn a_fulfilment_customer_can_be_read_changed_and_reported() {
    use picqer_rs::customers::Language;
    use picqer_rs::fulfilment::{
        FulfilmentCustomerChanges, FulfilmentCustomerName, NewFulfilmentCustomer, ReportPeriod,
    };

    let client = client();
    let customers = client.fulfilment_customers();
    let name = "picqer-rs-live-fulfilment";
    let all = customers.list_all().await.unwrap();
    let existing = all.iter().find(|customer| customer.name == name);
    let id = match existing {
        Some(customer) => customer.idfulfilment_customer,
        None => {
            customers
                .create(&NewFulfilmentCustomer::new(
                    FulfilmentCustomerName::new(name).unwrap(),
                    Language::Nl,
                ))
                .await
                .unwrap()
                .idfulfilment_customer
        }
    };

    let changed = customers
        .update(id, &FulfilmentCustomerChanges::new().language(Language::En))
        .await
        .unwrap();
    assert_eq!(changed.language.unwrap().known(), Some(&Language::En));
    let restored = customers
        .update(id, &FulfilmentCustomerChanges::new().language(Language::Nl))
        .await
        .unwrap();
    assert_eq!(restored.language.unwrap().known(), Some(&Language::Nl));
    assert_eq!(customers.get(id).await.unwrap().name, name);

    let report = customers
        .report(
            id,
            ReportPeriod::new()
                .from("2026-11-01".parse().unwrap())
                .until("2026-11-09".parse().unwrap()),
        )
        .await
        .unwrap();
    let meta = report.meta_data.unwrap();
    assert_eq!(meta.idfulfilment_customer, Some(id));
    assert_eq!(meta.start_date.unwrap().to_string(), "2026-11-01");
    assert!(report.inbound.is_some() && report.outbound.is_some());
    assert!(report.storage_usage.is_some() && report.returns.is_some());
}
