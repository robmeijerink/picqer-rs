mod common;

use common::{created, respond, setup};
use picqer_rs::Reported;
use picqer_rs::country::CountryCode;
use picqer_rs::customers::{
    AddressChanges, AddressName, CustomerChanges, CustomerName, CustomerNumber, CustomerQuery,
    DiscountPercentage, Language, NewCustomer, NewCustomerAddress,
};
use picqer_rs::pagination::Page;
use picqer_rs::types::{CustomerAddressId, CustomerId, TagId};
use serde_json::{Value, json};
use wiremock::matchers::{body_json, method, path, query_param, query_param_is_missing};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn customers_are_listed_and_filtered() {
    let (server, client) = setup().await;
    Mock::given(path("/customers"))
        .and(query_param("search", "Example"))
        .and(query_param("customerid", "EX-1"))
        .and(query_param_is_missing("offset"))
        .respond_with(respond("customers"))
        .expect(1)
        .mount(&server)
        .await;

    let query = CustomerQuery::new()
        .search("Example")
        .customerid(CustomerNumber::new("EX-1").unwrap());
    let customers = client.customers().list(&query, Page::FIRST).await.unwrap();
    assert_eq!(customers.len(), 2);
    assert_eq!(customers[0].name, "Example Customer 1");
    assert_eq!(customers[0].customerid, None);
    assert_eq!(customers[0].addresses.len(), 1);
    assert!(customers[0].tags.is_empty());
}

#[tokio::test]
async fn a_customer_with_every_field_is_read_completely() {
    let (server, client) = setup().await;
    Mock::given(path("/customers/5"))
        .respond_with(respond("customer_full"))
        .mount(&server)
        .await;

    let customer = client.customers().get(CustomerId(5)).await.unwrap();
    assert_eq!(customer.name, "Example Trading B.V.");
    assert!(customer.customerid.as_deref().unwrap().starts_with("EX-"));
    assert_eq!(customer.language, Some(Reported::Known(Language::En)));
    assert_eq!(customer.discount.unwrap().get(), 12.5);
    assert_eq!(customer.auto_split, Some(true));
    assert_eq!(customer.emailaddress.as_deref(), Some("jane@example.com"));
    assert_eq!(customer.tags["lightspeed"].title, "lightspeed");
    assert_eq!(customer.addresses.len(), 2);
    let home = &customer.addresses[0];
    assert_eq!(home.address2.as_deref(), Some("Unit 4"));
    assert_eq!(home.country, Some(Reported::Known(CountryCode::NL)));
    assert_eq!(home.defaultinvoice, Some(true));
}

#[tokio::test]
async fn what_picqer_stores_is_read_even_when_it_would_not_be_sent() {
    let (server, client) = setup().await;
    let mut customer = common::json("customer_full");
    customer["discount"] = json!(150);
    customer["language"] = json!("de");
    customer["addresses"][0]["country"] = json!("nl");
    Mock::given(path("/customers/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(customer))
        .mount(&server)
        .await;

    let customer = client.customers().get(CustomerId(5)).await.unwrap();
    assert_eq!(customer.discount.unwrap().get(), 150.0);
    assert_eq!(
        customer.language,
        Some(Reported::Unknown(Value::from("de")))
    );
    assert_eq!(
        customer.addresses[0].country,
        Some(Reported::Unknown(Value::from("nl")))
    );
}

#[tokio::test]
async fn a_customer_is_created_changed_and_deleted() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/customers"))
        .and(body_json(json!({
            "name": "Example Trading B.V.",
            "language": "en",
            "discount": 12.5,
            "addresses": [{"name": "Home", "country": "NL", "city": "Amsterdam"}]
        })))
        .respond_with(created("customer_full"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/customers/5"))
        .and(body_json(json!({"name": "Renamed", "calculatevat": false})))
        .respond_with(respond("customer_full"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/customers/5"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let new = NewCustomer::new(CustomerName::new("Example Trading B.V.").unwrap())
        .language(Language::En)
        .discount(DiscountPercentage::new(12.5).unwrap())
        .address(
            NewCustomerAddress::new(AddressName::new("Home").unwrap(), CountryCode::NL)
                .city("Amsterdam".to_owned()),
        );
    let customers = client.customers();
    customers.create(&new).await.unwrap();
    let changes = CustomerChanges::new()
        .name(CustomerName::new("Renamed").unwrap())
        .calculatevat(false);
    customers.update(CustomerId(5), &changes).await.unwrap();
    customers.delete(CustomerId(5)).await.unwrap();
}

#[tokio::test]
async fn a_customer_is_found_by_its_own_number() {
    let (server, client) = setup().await;
    Mock::given(path("/customers"))
        .and(query_param("customerid", "EX-1"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!([common::json("customer_full")])),
        )
        .expect(1)
        .mount(&server)
        .await;

    let number = CustomerNumber::new("EX-1").unwrap();
    assert!(
        client
            .customers()
            .find_by_number(&number)
            .await
            .unwrap()
            .is_some()
    );
}

#[tokio::test]
async fn addresses_are_listed_added_changed_and_removed() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/customers/5/addresses"))
        .respond_with(respond("customer_addresses"))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/customers/5/addresses"))
        .and(body_json(
            json!({"name": "Warehouse", "country": "NL", "defaultdelivery": true}),
        ))
        .respond_with(created("customer_address"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/customers/5/addresses/9"))
        .and(body_json(json!({"zipcode": "3000 AA"})))
        .respond_with(respond("customer_address"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/customers/5/addresses/9"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let customers = client.customers();
    assert_eq!(
        customers.addresses_all(CustomerId(5)).await.unwrap().len(),
        2
    );
    let new = NewCustomerAddress::new(AddressName::new("Warehouse").unwrap(), CountryCode::NL)
        .defaultdelivery(true);
    customers.add_address(CustomerId(5), &new).await.unwrap();
    let changes = AddressChanges::new().zipcode("3000 AA".to_owned());
    customers
        .update_address(CustomerId(5), CustomerAddressId(9), &changes)
        .await
        .unwrap();
    customers
        .remove_address(CustomerId(5), CustomerAddressId(9))
        .await
        .unwrap();
}

#[tokio::test]
async fn tags_of_a_customer() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/customers/5/tags"))
        .respond_with(respond("customer_tags"))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/customers/5/tags"))
        .and(body_json(json!({"idtag": 9})))
        .respond_with(created("tag"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/customers/5/tags/9"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let customers = client.customers();
    assert_eq!(customers.tags(CustomerId(5)).await.unwrap().len(), 1);
    customers.add_tag(CustomerId(5), TagId(9)).await.unwrap();
    customers.remove_tag(CustomerId(5), TagId(9)).await.unwrap();
}

#[tokio::test]
async fn customer_fields_follow_the_documented_example() {
    use picqer_rs::fields::FieldType;
    use picqer_rs::types::CustomerFieldId;

    let (server, client) = setup().await;
    let example = json!({
        "idcustomerfield": 103, "title": "Delivery options", "type": "text", "values": [], "required": true
    });
    Mock::given(path("/customerfields"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([example])))
        .mount(&server)
        .await;
    Mock::given(path("/customerfields/103"))
        .respond_with(ResponseTemplate::new(200).set_body_json(example))
        .mount(&server)
        .await;

    let all = client.customer_fields().list_all().await.unwrap();
    assert_eq!(all[0].field_type, Some(Reported::Known(FieldType::Text)));
    assert_eq!(all[0].required, Some(true));
    let one = client
        .customer_fields()
        .get(CustomerFieldId(103))
        .await
        .unwrap();
    assert_eq!(one, all[0]);
}

#[tokio::test]
async fn the_values_of_customer_fields_are_read_and_sent() {
    use picqer_rs::customers::{CustomerChanges, CustomerFieldValue};
    use picqer_rs::types::CustomerFieldId;

    let (server, client) = setup().await;
    let mut customer = common::json("customer_full");
    customer["customerfields"] = json!([{"idcustomerfield": 103, "title": "Sizes", "value": "L"}]);
    Mock::given(method("PUT"))
        .and(path("/customers/5"))
        .and(body_json(json!({"customerfields": [
            {"idcustomerfield": 103, "value": "L"},
            {"idcustomerfield": 104, "value": null}
        ]})))
        .respond_with(ResponseTemplate::new(200).set_body_json(customer))
        .expect(1)
        .mount(&server)
        .await;

    let changes = CustomerChanges::new().customerfields(vec![
        CustomerFieldValue::new(CustomerFieldId(103), "L"),
        CustomerFieldValue::cleared(CustomerFieldId(104)),
    ]);
    let updated = client
        .customers()
        .update(CustomerId(5), &changes)
        .await
        .unwrap();
    assert_eq!(updated.customerfields.len(), 1);
    assert_eq!(updated.customerfields[0].value.as_deref(), Some("L"));
}
