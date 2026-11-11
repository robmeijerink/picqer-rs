//! Makes a task and puts a comment on an order.
//!
//! Writes to the account. The task is deleted again, but Picqer cannot delete a comment. Pass
//! the id of an order.
//!
//! ```sh
//! cargo run --example tasks_and_comments -- <order id>
//! ```

mod common;

use picqer_rs::comments::{CommentBody, CommentTarget, NewComment};
use picqer_rs::tasks::{NewTask, TaskTitle};
use picqer_rs::types::OrderId;

#[tokio::main]
async fn main() -> picqer_rs::Result<()> {
    let id: u64 = std::env::args()
        .nth(1)
        .and_then(|id| id.parse().ok())
        .expect("an order id");
    let client = common::client();

    let task = client
        .tasks()
        .create(
            &NewTask::new(TaskTitle::new("Check the picqer-rs example")?).description("Details"),
        )
        .await?;
    println!("task {} is {:?}", task.idtask, task.title);
    client.tasks().complete(task.idtask).await?;
    client.tasks().uncomplete(task.idtask).await?;
    client.tasks().delete(task.idtask).await?;

    let target = CommentTarget::Order(OrderId(id));
    let comment = NewComment::new(CommentBody::new("Added by the picqer-rs example")?);
    client.comments().add(target, &comment).await?;
    for comment in client.comments().for_target_all(target).await? {
        println!("{comment:?}");
    }

    Ok(())
}
