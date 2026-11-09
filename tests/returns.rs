mod common;

use std::num::NonZeroU32;

use common::{respond, setup};
use picqer_rs::Reported;
use picqer_rs::country::CountryCode;
use picqer_rs::customers::Language;
use picqer_rs::pagination::Page;
use picqer_rs::products::LotNumber;
use picqer_rs::returns::{
    NewReplacementProduct, NewReturn, NewReturnLog, NewReturnedProduct, ReceiveReturn,
    ReceivedProduct, ReplacementProductChanges, ReturnChanges, ReturnLogType, ReturnQuery,
    ReturnStatusFilter, ReturnedProductChanges, ReturnedProductStatus,
};
use picqer_rs::types::{
    EmailAddress, LocationId, Money, NonEmpty, ProductId, ReplacementProductId, ReturnId,
    ReturnReasonId, ReturnStatusId, ReturnedProductId, WarehouseId,
};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn returns_are_listed_without_their_products() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/returns"))
        .respond_with(respond("returns"))
        .expect(1)
        .mount(&server)
        .await;

    let returns = client
        .returns()
        .list(&ReturnQuery::new(), Page::FIRST)
        .await
        .unwrap();
    assert_eq!(returns.len(), 4);
    let first = &returns[0];
    assert!(first.returnid.as_deref().unwrap().starts_with('R'));
    assert_eq!(first.country, Some(Reported::Known(CountryCode::NL)));
    assert_eq!(first.language, None);
    assert_eq!(returns[1].language, Some(Reported::Known(Language::Nl)));
    assert!(first.created_at.is_some());
    assert!(first.returned_products.is_empty());
    assert!(first.return_status.is_none());
    assert_eq!(first.received_at, None);
}

#[tokio::test]
async fn a_single_return_comes_with_everything_around_it() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/returns/5"))
        .respond_with(respond("return"))
        .mount(&server)
        .await;

    let one = client.returns().get(ReturnId(5)).await.unwrap();
    let status = one.return_status.as_ref().unwrap();
    assert_eq!(status.idreturn_status, one.idreturn_status);
    assert_eq!(status.completed, Some(false));
    let customer = one.customer.as_ref().unwrap();
    assert!(customer.name.is_some());
    let order = one.order.as_ref().unwrap();
    assert!(order.orderid.as_deref().unwrap().starts_with('O'));
    assert_eq!(one.returned_products.len(), 1);
    let line = &one.returned_products[0];
    assert_eq!(
        line.status,
        Some(Reported::Known(ReturnedProductStatus::Expected))
    );
    assert_eq!(line.changeable, Some(true));
    assert!(line.price.unwrap() > Money::from_cents(0).unwrap());
    assert_eq!(line.location, None);
    assert!(line.return_reason.as_ref().unwrap().name.is_some());
    assert_eq!(one.replacement_products.len(), 2);
    let replacement = &one.replacement_products[0];
    assert_eq!(replacement.status.as_deref(), Some("concept"));
    assert_eq!(replacement.idpicklist, None);
    let totals = one.totals.as_ref().unwrap();
    assert!(totals.returned_products_count.unwrap() >= 1);
}

#[tokio::test]
async fn a_product_that_is_added_to_the_stock_says_where() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/returns/6"))
        .respond_with(respond("return_with_stock"))
        .mount(&server)
        .await;

    let one = client.returns().get(ReturnId(6)).await.unwrap();
    let added = one
        .returned_products
        .iter()
        .find(|line| {
            line.status == Some(Reported::Known(ReturnedProductStatus::AddedToStock))
                && line.lot_number.is_some()
        })
        .expect("a product that is added to the stock with a lot");
    assert_eq!(added.changeable, Some(false));
    assert!(added.idwarehouse.is_some());
    assert_eq!(added.lot_number.as_deref(), Some("R1"));
    assert!(added.expiration_date.is_some());
    let without_location = one
        .returned_products
        .iter()
        .find(|line| {
            line.status == Some(Reported::Known(ReturnedProductStatus::AddedToStock))
                && line.lot_number.is_none()
        })
        .unwrap();
    assert_eq!(without_location.idlocation, None);
    assert!(without_location.idwarehouse.is_some());
}

#[tokio::test]
async fn filters_become_query_parameters() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/returns"))
        .and(query_param("search", "R2026"))
        .and(query_param("status", "completed"))
        .and(query_param("emailaddress", "a@example.com"))
        .and(query_param("idcustomer", "5"))
        .and(query_param("created_after", "2026-11-09 16:00:00"))
        .and(query_param("updated_after", "2026-11-09 17:00:00"))
        .respond_with(respond("returns"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/returns"))
        .and(query_param("status", "17650"))
        .respond_with(respond("returns"))
        .expect(1)
        .mount(&server)
        .await;

    let query = ReturnQuery::new()
        .search("R2026")
        .status(ReturnStatusFilter::Completed)
        .emailaddress("a@example.com")
        .idcustomer(picqer_rs::types::CustomerId(5))
        .created_after("2026-11-09 16:00:00".parse().unwrap())
        .updated_after("2026-11-09 17:00:00".parse().unwrap());
    client.returns().list(&query, Page::FIRST).await.unwrap();
    client
        .returns()
        .list(
            &ReturnQuery::new().status(ReturnStatusFilter::Status(ReturnStatusId(17650))),
            Page::FIRST,
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn a_return_is_made_changed_and_deleted() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/returns"))
        .and(body_json(json!({
            "idreturn_status": 17648, "name": "John Hendriks", "country": "NL",
            "emailaddress": "john@example.org", "language": "nl",
            "returned_products": [{"idreturn_reason": 13906, "idproduct": 3, "amount": 2}],
            "replacement_products": [{"idproduct": 4, "amount": 1, "price": 9.5}]
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(common::json("return")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/returns/5"))
        .and(body_json(
            json!({"name": null, "tracking_code": "TRK1", "idreturn_status": 17650}),
        ))
        .respond_with(respond("return"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/returns/5"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let returns = client.returns();
    let new = NewReturn::new(ReturnStatusId(17648))
        .name("John Hendriks")
        .country(CountryCode::NL)
        .emailaddress(EmailAddress::new("john@example.org").unwrap())
        .language(Language::Nl)
        .returned_product(NewReturnedProduct::new(
            ReturnReasonId(13906),
            ProductId(3),
            NonZeroU32::new(2).unwrap(),
        ))
        .replacement_product(
            NewReplacementProduct::new(ProductId(4), NonZeroU32::new(1).unwrap())
                .price(Money::from_cents(950).unwrap()),
        );
    let made = returns.create(&new).await.unwrap();
    returns
        .update(
            ReturnId(5),
            &ReturnChanges::new()
                .without_name()
                .tracking_code("TRK1")
                .idreturn_status(ReturnStatusId(17650)),
        )
        .await
        .unwrap();
    assert!(made.idreturn.0 > 0);
    returns.delete(ReturnId(5)).await.unwrap();
}

#[tokio::test]
async fn returned_and_replacement_products_are_listed_added_changed_and_removed() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/returns/5/returned_products"))
        .respond_with(respond("return_returned_products"))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/returns/5/returned_products"))
        .and(body_json(
            json!({"idreturn_reason": 13906, "idproduct": 3, "amount": 1, "price": 12.95}),
        ))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(common::json("return_returned_product")),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/returns/5/returned_products/11"))
        .and(body_json(
            json!({"price": 20.0, "amount": 3, "idreturn_reason": 13907}),
        ))
        .respond_with(respond("return_returned_product"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/returns/5/returned_products/11"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/returns/5/replacement_products"))
        .respond_with(respond("return_replacement_products"))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/returns/5/replacement_products"))
        .and(body_json(json!({"idproduct": 4, "amount": 2})))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(common::json("return_replacement_product")),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/returns/5/replacement_products/22"))
        .and(body_json(json!({"price": 5.0, "amount": 3})))
        .respond_with(respond("return_replacement_product"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/returns/5/replacement_products/22"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let returns = client.returns();
    let id = ReturnId(5);
    let returned = returns.returned_products(id).await.unwrap();
    assert_eq!(returned.len(), 1);
    returns
        .add_returned_product(
            id,
            NewReturnedProduct::new(
                ReturnReasonId(13906),
                ProductId(3),
                NonZeroU32::new(1).unwrap(),
            )
            .price(Money::from_cents(1295).unwrap()),
        )
        .await
        .unwrap();
    returns
        .update_returned_product(
            id,
            ReturnedProductId(11),
            ReturnedProductChanges::new()
                .price(Money::from_cents(2000).unwrap())
                .amount(NonZeroU32::new(3).unwrap())
                .idreturn_reason(ReturnReasonId(13907)),
        )
        .await
        .unwrap();
    returns
        .remove_returned_product(id, ReturnedProductId(11))
        .await
        .unwrap();
    assert_eq!(returns.replacement_products(id).await.unwrap().len(), 2);
    returns
        .add_replacement_product(
            id,
            NewReplacementProduct::new(ProductId(4), NonZeroU32::new(2).unwrap()),
        )
        .await
        .unwrap();
    returns
        .update_replacement_product(
            id,
            ReplacementProductId(22),
            ReplacementProductChanges::new()
                .price(Money::from_cents(500).unwrap())
                .amount(NonZeroU32::new(3).unwrap()),
        )
        .await
        .unwrap();
    returns
        .remove_replacement_product(id, ReplacementProductId(22))
        .await
        .unwrap();
}

#[tokio::test]
async fn the_logbook_is_read_and_comments_and_statuses_are_added() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/returns/5/logs"))
        .respond_with(respond("return_logs"))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/returns/5/logs"))
        .and(body_json(json!({
            "idreturn_status": 17650, "message": "Label sent", "notify_customer": true
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(common::json("return_log")))
        .expect(1)
        .mount(&server)
        .await;

    let logs = client.returns().logs(ReturnId(5)).await.unwrap();
    assert!(logs.len() >= 2);
    let types: Vec<_> = logs.iter().filter_map(|log| log.log_type.clone()).collect();
    assert!(types.contains(&Reported::Known(ReturnLogType::Created)));
    assert!(types.contains(&Reported::Known(ReturnLogType::Comment)));
    let created = logs
        .iter()
        .find(|log| log.log_type == Some(Reported::Known(ReturnLogType::Created)))
        .unwrap();
    assert_eq!(created.notified_customer, Some(false));
    assert!(created.user.is_some());
    let comment = logs
        .iter()
        .find(|log| log.log_type == Some(Reported::Known(ReturnLogType::Comment)))
        .unwrap();
    assert_eq!(
        comment.user.as_ref().unwrap().name.as_deref(),
        Some("Alex Example")
    );
    assert!(comment.return_status.as_ref().unwrap().name.is_some());

    let added = client
        .returns()
        .add_log(
            ReturnId(5),
            &NewReturnLog::comment("Label sent")
                .with_status(ReturnStatusId(17650))
                .notify_customer(),
        )
        .await
        .unwrap();
    assert_eq!(
        added.log_type,
        Some(Reported::Known(ReturnLogType::Comment))
    );
}

#[tokio::test]
async fn products_are_received_with_a_warehouse_when_they_go_to_the_stock() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/returns/5/receive"))
        .and(body_json(json!({
            "idwarehouse": 14153,
            "return_products": [
                {
                    "idreturn_product": 21, "status": "added_to_stock", "idlocation": 6516,
                    "lot_number": "R1", "expiration_date": "2027-01-15"
                },
                {"idreturn_product": 22, "status": "unsalable"}
            ]
        })))
        .respond_with(respond("return_received"))
        .expect(1)
        .mount(&server)
        .await;

    let products = NonEmpty::new(vec![
        ReceivedProduct::new(ReturnedProductId(21), ReturnedProductStatus::AddedToStock)
            .idlocation(LocationId(6516))
            .lot_number(LotNumber::new("R1").unwrap())
            .expiration_date("2027-01-15".parse().unwrap()),
        ReceivedProduct::new(ReturnedProductId(22), ReturnedProductStatus::Unsalable),
    ])
    .unwrap();
    let refused = client
        .returns()
        .receive(ReturnId(5), &ReceiveReturn::new(products.clone()))
        .await
        .unwrap_err();
    assert!(matches!(refused, picqer_rs::Error::Validation(_)));
    let received = client
        .returns()
        .receive(
            ReturnId(5),
            &ReceiveReturn::new(products).idwarehouse(WarehouseId(14153)),
        )
        .await
        .unwrap();
    assert!(received.full_address.is_some());
}

#[tokio::test]
async fn the_lot_errors_of_each_line_are_in_the_context_of_the_error() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/returns/5/receive"))
        .respond_with(ResponseTemplate::new(422).set_body_json(json!({
            "error": true, "error_code": 46,
            "error_message": "Check the lot of the marked products.",
            "context": {"errors": {"21": {"lot_number": ["The lot number is required."]}}}
        })))
        .mount(&server)
        .await;

    let products = NonEmpty::new(vec![ReceivedProduct::new(
        ReturnedProductId(21),
        ReturnedProductStatus::AddedToStock,
    )])
    .unwrap();
    let refused = client
        .returns()
        .receive(
            ReturnId(5),
            &ReceiveReturn::new(products).idwarehouse(WarehouseId(14153)),
        )
        .await
        .unwrap_err();
    let api = refused.as_api().unwrap();
    assert_eq!(api.status, 422);
    assert_eq!(
        api.context.as_ref().unwrap()["errors"]["21"]["lot_number"][0],
        "The lot number is required."
    );
}

#[tokio::test]
async fn the_statuses_and_the_reasons_are_listed() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/return_statuses"))
        .respond_with(respond("return_statuses"))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/return_reasons"))
        .respond_with(respond("return_reasons"))
        .mount(&server)
        .await;

    let statuses = client.return_statuses().list().await.unwrap();
    assert_eq!(statuses.len(), 4);
    assert!(statuses.iter().any(|status| status.default == Some(true)));
    assert!(statuses.iter().any(|status| status.completed == Some(true)));
    assert!(statuses.iter().all(|status| status.deleted_at.is_none()));
    let reasons = client.return_reasons().list().await.unwrap();
    assert_eq!(reasons.len(), 4);
    assert!(reasons.iter().any(|reason| reason.default == Some(true)));
}
