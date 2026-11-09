mod common;

use common::{respond, setup};
use picqer_rs::pagination::Page;
use picqer_rs::tasks::{NewTask, TaskChanges, TaskQuery, TaskTitle};
use picqer_rs::types::{TaskId, UserId};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn tasks_are_listed_with_their_deadline_and_assignee() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/tasks"))
        .respond_with(respond("tasks"))
        .expect(1)
        .mount(&server)
        .await;

    let tasks = client
        .tasks()
        .list(TaskQuery::new(), Page::FIRST)
        .await
        .unwrap();
    assert_eq!(tasks.len(), 2);
    let assigned = tasks
        .iter()
        .find(|task| task.title == "Restock shelf")
        .unwrap();
    assert_eq!(assigned.assigned_to_iduser, Some(UserId(1)));
    assert_eq!(assigned.is_seen_by_assignee, Some(false));
    assert_eq!(
        assigned.deadline_date.unwrap().to_string(),
        "2031-03-04 00:00:00"
    );
    assert_eq!(assigned.deadline_time.unwrap().to_string(), "23:59:00");
    assert_eq!(assigned.created_by_iduser, Some(UserId(1)));
    assert_eq!(assigned.completed_at, None);
    let plain = tasks
        .iter()
        .find(|task| task.title == "picqer-rs fixture plain")
        .unwrap();
    assert_eq!(plain.description, None);
    assert_eq!(plain.assigned_to_iduser, None);
    assert_eq!(plain.is_seen_by_assignee, None);
    assert_eq!(plain.deadline_date, None);
}

#[tokio::test]
async fn the_filter_becomes_a_query_parameter() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/tasks"))
        .and(query_param("completed", "true"))
        .respond_with(respond("tasks"))
        .expect(1)
        .mount(&server)
        .await;

    client
        .tasks()
        .list_all(TaskQuery::new().completed(true))
        .await
        .unwrap();
}

#[tokio::test]
async fn a_task_is_made_completed_and_deleted() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/tasks"))
        .and(body_json(json!({
            "title": "Restock shelf", "description": "Refill", "deadline_date": "2031-03-04",
            "deadline_time": "23:59:00", "assigned_to_iduser": 1
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(common::json("task")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/tasks/5/complete"))
        .respond_with(respond("task_completed"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/tasks/5/uncomplete"))
        .respond_with(respond("task_plain"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/tasks/5"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let tasks = client.tasks();
    let made = tasks
        .create(
            &NewTask::new(TaskTitle::new("Restock shelf").unwrap())
                .description("Refill")
                .deadline("2031-03-04".parse().unwrap(), "23:59:00".parse().unwrap())
                .assigned_to(UserId(1)),
        )
        .await
        .unwrap();
    assert_eq!(made.title, "Restock shelf");
    let done = tasks.complete(TaskId(5)).await.unwrap();
    assert!(done.completed_at.is_some());
    assert_eq!(done.completed_by_iduser, Some(UserId(1)));
    assert_eq!(
        tasks.uncomplete(TaskId(5)).await.unwrap().completed_at,
        None
    );
    tasks.delete(TaskId(5)).await.unwrap();
}

#[tokio::test]
async fn a_change_without_an_assignee_keeps_the_assignee_of_the_task() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/tasks/5"))
        .respond_with(respond("task"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/tasks/5"))
        .and(body_json(
            json!({"title": "Renamed", "assigned_to_iduser": 1}),
        ))
        .respond_with(respond("task"))
        .expect(1)
        .mount(&server)
        .await;

    client
        .tasks()
        .update(
            TaskId(5),
            &TaskChanges::new().title(TaskTitle::new("Renamed").unwrap()),
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn naming_the_assignee_needs_no_extra_call() {
    let (server, client) = setup().await;
    Mock::given(method("PUT"))
        .and(path("/tasks/5"))
        .and(body_json(
            json!({"description": null, "assigned_to_iduser": null}),
        ))
        .respond_with(respond("task_plain"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/tasks/5"))
        .and(body_json(json!({"assigned_to_iduser": 2})))
        .respond_with(respond("task"))
        .expect(1)
        .mount(&server)
        .await;

    let tasks = client.tasks();
    tasks
        .update(
            TaskId(5),
            &TaskChanges::new().without_description().unassigned(),
        )
        .await
        .unwrap();
    tasks
        .update(TaskId(5), &TaskChanges::new().assigned_to(UserId(2)))
        .await
        .unwrap();
}

#[tokio::test]
async fn the_errors_of_a_task_come_back_as_an_errors_object() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/tasks/1"))
        .respond_with(
            ResponseTemplate::new(404).set_body_json(json!({"errors": {"task": "Task not found"}})),
        )
        .mount(&server)
        .await;

    let missing = client.tasks().get(TaskId(1)).await.unwrap_err();
    assert!(missing.is_not_found());
    assert_eq!(
        missing.as_api().unwrap().errors,
        Some(json!({"task": "Task not found"}))
    );
}
