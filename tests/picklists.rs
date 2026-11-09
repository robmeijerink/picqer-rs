mod common;

use std::num::NonZeroU32;

use common::{created, respond, setup};
use picqer_rs::Reported;
use picqer_rs::pagination::Page;
use picqer_rs::picklists::{
    AdviceSource, CancelProduct, Pick, PickLocationAmount, PickTarget, PicklistChanges,
    PicklistQuery, PicklistStatus, Split, SplitLine,
};
use picqer_rs::types::{
    DateTime, LocationId, NonEmpty, PackagingId, PauseReason, PicklistId, PicklistNumber,
    PicklistProductId, ProductCode, UserId,
};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

fn amount(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value).unwrap()
}

#[tokio::test]
async fn picklists_are_listed_and_filtered() {
    let (server, client) = setup().await;
    Mock::given(path("/picklists"))
        .and(query_param("status", "new"))
        .and(query_param("assigned_to_iduser", "43968"))
        .respond_with(respond("picklists"))
        .expect(1)
        .mount(&server)
        .await;

    let query = PicklistQuery::new()
        .status(PicklistStatus::New)
        .assigned_to(UserId(43968));
    let picklists = client.picklists().list(&query, Page::FIRST).await.unwrap();
    assert_eq!(picklists.len(), 2);
    assert_eq!(picklists[0].status, Reported::Known(PicklistStatus::New));
    assert!(picklists[0].picklistid.starts_with('P'));
}

#[tokio::test]
async fn a_picklist_is_read_with_its_lines_and_pick_locations() {
    let (server, client) = setup().await;
    Mock::given(path("/picklists/5"))
        .respond_with(respond("picklist"))
        .mount(&server)
        .await;

    let picklist = client.picklists().get(PicklistId(5)).await.unwrap();
    assert_eq!(picklist.products.len(), 2);
    assert_eq!(picklist.totalproducts, 5);
    assert_eq!(picklist.totalpicked, 0);
    let line = &picklist.products[0];
    assert_eq!(line.amount, 3);
    assert_eq!(line.amount_picked, 0);
    assert!(
        picklist
            .products
            .iter()
            .all(|line| !line.pick_locations.is_empty())
    );
    let parts: Vec<_> = picklist
        .products
        .iter()
        .flat_map(|line| &line.pick_locations)
        .collect();
    let unspecific = parts.iter().find(|part| part.idlocation.is_none()).unwrap();
    assert_eq!(unspecific.name, None);
    let located = parts.iter().find(|part| part.idlocation.is_some()).unwrap();
    assert!(located.name.is_some());
    assert_eq!(picklist.assigned_to_iduser, None);
    assert_eq!(picklist.idpicklist_batch, None);
}

#[tokio::test]
async fn assigned_snoozed_and_paused_picklists_report_why() {
    let (server, client) = setup().await;
    for (name, fixture) in [
        ("1", "picklist_assigned"),
        ("2", "picklist_snoozed"),
        ("3", "picklist_paused"),
    ] {
        Mock::given(path(format!("/picklists/{name}")))
            .respond_with(respond(fixture))
            .mount(&server)
            .await;
    }

    let picklists = client.picklists();
    let assigned = picklists.get(PicklistId(1)).await.unwrap();
    assert_eq!(assigned.assigned_to_iduser, Some(UserId(43968)));
    let snoozed = picklists.get(PicklistId(2)).await.unwrap();
    assert_eq!(snoozed.status, Reported::Known(PicklistStatus::Snoozed));
    assert_eq!(
        snoozed.snoozed_until.unwrap().to_string(),
        "2030-01-01 12:00:00"
    );
    let paused = picklists.get(PicklistId(3)).await.unwrap();
    assert_eq!(paused.status, Reported::Known(PicklistStatus::Paused));
    assert_eq!(paused.paused_reason.as_deref(), Some("Wrong address"));
}

#[tokio::test]
async fn a_picklist_is_found_by_its_number() {
    let (server, client) = setup().await;
    Mock::given(path("/picklists"))
        .and(query_param("picklistid", "P2026-1370"))
        .respond_with(respond("picklists"))
        .expect(1)
        .mount(&server)
        .await;

    let number = PicklistNumber::new("P2026-1370").unwrap();
    assert!(
        client
            .picklists()
            .find_by_number(&number)
            .await
            .unwrap()
            .is_some()
    );
}

#[tokio::test]
async fn a_picklist_is_changed_and_every_action_is_a_post_to_its_own_path() {
    let (server, client) = setup().await;
    Mock::given(method("PUT"))
        .and(path("/picklists/5"))
        .and(body_json(json!({"urgent": true})))
        .respond_with(respond("picklist"))
        .expect(1)
        .mount(&server)
        .await;
    for action in ["close", "pickall", "unassign", "resume"] {
        Mock::given(method("POST"))
            .and(path(format!("/picklists/5/{action}")))
            .respond_with(respond("picklist"))
            .expect(1)
            .mount(&server)
            .await;
    }

    let picklists = client.picklists();
    let id = PicklistId(5);
    picklists
        .update(id, &PicklistChanges::new().urgent(true))
        .await
        .unwrap();
    picklists.close(id).await.unwrap();
    picklists.pick_all(id).await.unwrap();
    picklists.unassign(id).await.unwrap();
    picklists.resume(id).await.unwrap();
}

#[tokio::test]
async fn assigning_snoozing_and_pausing_send_their_own_bodies() {
    let (server, client) = setup().await;
    let bodies = [
        ("assign", json!({"iduser": 43968})),
        ("snooze", json!({"snooze_until": "2030-01-01 12:00:00"})),
        ("pause", json!({"reason": "Wrong address"})),
    ];
    for (action, body) in &bodies {
        Mock::given(method("POST"))
            .and(path(format!("/picklists/5/{action}")))
            .and(body_json(body))
            .respond_with(respond("picklist"))
            .expect(1)
            .mount(&server)
            .await;
    }
    Mock::given(method("POST"))
        .and(path("/picklists/6/snooze"))
        .and(body_json(json!({"snooze_until": null})))
        .respond_with(respond("picklist"))
        .expect(1)
        .mount(&server)
        .await;

    let picklists = client.picklists();
    picklists
        .assign(PicklistId(5), UserId(43968))
        .await
        .unwrap();
    let until: DateTime = "2030-01-01 12:00:00".parse().unwrap();
    picklists.snooze(PicklistId(5), until).await.unwrap();
    picklists.unsnooze(PicklistId(6)).await.unwrap();
    let reason = PauseReason::new("Wrong address").unwrap();
    picklists.pause(PicklistId(5), &reason).await.unwrap();
}

#[tokio::test]
async fn products_are_picked_and_unpicked() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/picklists/5/pick"))
        .and(body_json(json!({"product": "A-1", "amount": 2})))
        .respond_with(respond("picklist"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/picklists/5/pick"))
        .and(body_json(json!({"idpicklist_product": 7, "amount": -1})))
        .respond_with(respond("picklist"))
        .expect(1)
        .mount(&server)
        .await;

    let picklists = client.picklists();
    let code = ProductCode::new("A-1").unwrap();
    picklists
        .pick(
            PicklistId(5),
            &Pick::new(PickTarget::Product(code), amount(2)),
        )
        .await
        .unwrap();
    let undo = Pick::undo(PickTarget::Line(PicklistProductId(7)), amount(1));
    picklists.pick(PicklistId(5), &undo).await.unwrap();
}

#[tokio::test]
async fn the_pick_locations_of_a_line_are_changed() {
    let (server, client) = setup().await;
    Mock::given(method("PUT"))
        .and(path("/picklists/5/products/7/pick_locations"))
        .and(body_json(json!([{"idlocation": 10, "amount": 2}])))
        .respond_with(respond("picked_at"))
        .expect(1)
        .mount(&server)
        .await;

    let locations =
        NonEmpty::new(vec![PickLocationAmount::new(LocationId(10), amount(2))]).unwrap();
    let picked = client
        .picklists()
        .set_pick_locations(PicklistId(5), PicklistProductId(7), &locations)
        .await
        .unwrap();
    assert_eq!(picked.len(), 1);
    assert_eq!(picked[0].amount, 2);
}

#[tokio::test]
async fn a_picklist_is_split_and_a_product_is_cancelled() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/picklists/5/split"))
        .and(body_json(
            json!({"products": [{"idpicklist_product": 4821, "amount": 1}]}),
        ))
        .respond_with(created("picklist_split"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/picklists/5/cancelproduct"))
        .and(body_json(json!({"product": "A-1", "amount": 1})))
        .respond_with(respond("picklist"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/picklists/5/cancel"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let picklists = client.picklists();
    let lines = NonEmpty::new(vec![SplitLine::new(PicklistProductId(4821), amount(1))]).unwrap();
    let new = picklists
        .split(PicklistId(5), &Split::new(lines))
        .await
        .unwrap();
    assert_eq!(new.products.len(), 1);
    assert_ne!(new.idpicklist, PicklistId(5));
    let code = ProductCode::new("A-1").unwrap();
    picklists
        .cancel_product(PicklistId(5), &CancelProduct::product(code, amount(1)))
        .await
        .unwrap();
    picklists.cancel(PicklistId(5)).await.unwrap();
}

#[tokio::test]
async fn packaging_advice_is_empty_until_there_is_some() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/picklists/5/packaging-advice"))
        .respond_with(respond("packaging_advice"))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/picklists/5/packaging-advice"))
        .and(body_json(json!({"idpackaging": 34})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "idpackaging": 34, "source": "api",
            "created_at": "2025-06-25 14:21:14", "updated_at": "2025-06-25 14:39:43"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let none = client
        .picklists()
        .packaging_advice(PicklistId(5))
        .await
        .unwrap();
    assert_eq!(none.idpackaging, None);
    assert_eq!(none.source, None);
    let set = client
        .picklists()
        .set_packaging_advice(PicklistId(5), PackagingId(34))
        .await
        .unwrap();
    assert_eq!(set.source, Some(Reported::Known(AdviceSource::Api)));
}

#[tokio::test]
async fn the_pdfs_are_decoded() {
    let (server, client) = setup().await;
    for document in ["picklistpdf", "packinglistpdf"] {
        Mock::given(path(format!("/picklists/5/{document}")))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"idpicklist": 5, "pdf": "JVBERi0xLjcK"})),
            )
            .expect(1)
            .mount(&server)
            .await;
    }

    let picklists = client.picklists();
    assert_eq!(
        picklists.picklist_pdf(PicklistId(5)).await.unwrap(),
        b"%PDF-1.7\n"
    );
    assert!(
        picklists
            .packinglist_pdf(PicklistId(5))
            .await
            .unwrap()
            .starts_with(b"%PDF")
    );
}

#[tokio::test]
async fn a_pdf_that_is_not_base64_is_a_decode_error() {
    let (server, client) = setup().await;
    Mock::given(path("/picklists/5/picklistpdf"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"idpicklist": 5, "pdf": "***"})),
        )
        .mount(&server)
        .await;

    let error = client
        .picklists()
        .picklist_pdf(PicklistId(5))
        .await
        .unwrap_err();
    assert!(matches!(error, picqer_rs::Error::Decode { .. }));
}
