mod common;

use common::{respond, setup};
use picqer_rs::comments::{
    CommentBody, CommentQuery, CommentSource, CommentTarget, Commenter, NewComment, Person,
};
use picqer_rs::pagination::Page;
use picqer_rs::types::{
    CommentId, CustomerId, FulfilmentCustomerId, OrderId, PicklistBatchId, ProductId, UserId,
};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param, query_param_is_missing};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn comments_on_different_resources_have_their_own_source() {
    let (server, client) = setup().await;
    Mock::given(path("/comments"))
        .and(query_param_is_missing("idauthor"))
        .respond_with(respond("comments"))
        .expect(1)
        .mount(&server)
        .await;

    let comments = client
        .comments()
        .list(&CommentQuery::new(), Page::FIRST)
        .await
        .unwrap();
    assert_eq!(comments.len(), 3);
    assert!(matches!(
        &comments[0].source,
        CommentSource::Order { orderid, .. } if orderid.starts_with('O')
    ));
    assert!(matches!(
        &comments[1].source,
        CommentSource::Picklist { .. }
    ));
    assert!(matches!(
        &comments[2].source,
        CommentSource::Customer {
            customerid: None,
            ..
        }
    ));
    let Person::User(author) = &comments[0].author else {
        panic!("a user wrote the comment");
    };
    assert_eq!(author.full_name, "Alice Example");
    assert_eq!(comments[0].created_at.to_string().len(), 19);
}

#[tokio::test]
async fn a_comment_on_a_product_names_the_product() {
    let (server, client) = setup().await;
    Mock::given(path("/comments/1"))
        .respond_with(respond("comment_product"))
        .mount(&server)
        .await;

    let comment = client.comments().get(CommentId(1)).await.unwrap();
    assert!(matches!(
        &comment.source,
        CommentSource::Product { productcode, name, .. } if !productcode.is_empty() && !name.is_empty()
    ));
    assert!(comment.mentions.is_empty());
}

#[tokio::test]
async fn mentions_say_who_and_which_text_caused_them() {
    let (server, client) = setup().await;
    Mock::given(path("/comments/2"))
        .respond_with(respond("comment_mention"))
        .mount(&server)
        .await;

    let comment = client.comments().get(CommentId(2)).await.unwrap();
    assert_eq!(comment.show_at_related, Some(false));
    assert_eq!(comment.is_visible_fulfilment, Some(true));
    assert_eq!(comment.mentions.len(), 1);
    assert_eq!(comment.mentions[0].text, "@Alice Example");
    assert!(
        matches!(&comment.mentions[0].mentioned, Person::User(user) if user.iduser == UserId(43968))
    );
}

#[tokio::test]
async fn a_fulfilment_customer_can_be_an_author() {
    let (server, client) = setup().await;
    let mut comment = common::json("comment_customer");
    comment["author_type"] = json!("fulfilment_customer");
    comment["author"] = json!({"idfulfilment_customer": 11012, "username": "shop", "name": "Shop"});
    Mock::given(path("/comments/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(comment))
        .mount(&server)
        .await;

    let comment = client.comments().get(CommentId(3)).await.unwrap();
    assert!(matches!(
        &comment.author,
        Person::FulfilmentCustomer(customer) if customer.idfulfilment_customer == FulfilmentCustomerId(11012)
    ));
}

#[tokio::test]
async fn filters_name_the_type_of_a_fulfilment_customer() {
    let (server, client) = setup().await;
    Mock::given(path("/comments"))
        .and(query_param("idauthor", "11012"))
        .and(query_param("author_type", "fulfilment_customer"))
        .and(query_param("idmentioned", "7"))
        .and(query_param_is_missing("mentioned_type"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let query = CommentQuery::new()
        .author(Commenter::FulfilmentCustomer(FulfilmentCustomerId(11012)))
        .mentioned(Commenter::User(UserId(7)));
    assert!(client.comments().list_all(&query).await.unwrap().is_empty());
}

#[tokio::test]
async fn comments_are_listed_and_added_per_resource() {
    let (server, client) = setup().await;
    for resource in ["orders/5", "picklists/batches/9", "customers/4"] {
        Mock::given(method("GET"))
            .and(path(format!("/{resource}/comments")))
            .respond_with(respond("comments"))
            .expect(1)
            .mount(&server)
            .await;
    }
    Mock::given(method("POST"))
        .and(path("/products/6/comments"))
        .and(body_json(
            json!({"body": "Hello", "show_at_related": false}),
        ))
        .respond_with(respond("comment_product"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/comments/1"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let comments = client.comments();
    for target in [
        CommentTarget::Order(OrderId(5)),
        CommentTarget::PicklistBatch(PicklistBatchId(9)),
        CommentTarget::Customer(CustomerId(4)),
    ] {
        assert_eq!(comments.for_target_all(target).await.unwrap().len(), 3);
    }
    let new = NewComment::new(CommentBody::new("Hello").unwrap()).show_at_related(false);
    comments
        .add(CommentTarget::Product(ProductId(6)), &new)
        .await
        .unwrap();
    comments.delete(CommentId(1)).await.unwrap();
}

#[tokio::test]
async fn the_number_of_a_batch_is_a_number_in_a_comment_on_it() {
    let (server, client) = setup().await;
    Mock::given(path("/comments/2"))
        .respond_with(respond("comment_picklist_batch"))
        .mount(&server)
        .await;

    let comment = client.comments().get(CommentId(2)).await.unwrap();
    assert!(matches!(
        &comment.source,
        CommentSource::PicklistBatch { idpicklist_batch, picklist_batchid }
            if *idpicklist_batch == PicklistBatchId(9773319) && picklist_batchid == "5"
    ));
}
