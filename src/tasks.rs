//! Tasks.
//!
//! Tasks tell people in the warehouse what has to be done. A task can be assigned to a user and can
//! have a deadline.
//!
//! The documentation describes the fields and its examples show the same ones. What a developer
//! account also reports but the documentation never shows, such as `idrecurring_task`, is not read.

use std::fmt;
use std::str::FromStr;

use reqwest::Method;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value, json};
use time::format_description::FormatItem;
use time::macros::format_description;

use crate::client::Client;
use crate::error::{Result, ValidationError};
use crate::http::{Request, segments};
use crate::ops::{Call, NoContent};
use crate::pagination::Page;
use crate::types::{Date, DateTime, TaskId, UserId};

bounded_string! {
    /// The title of a task. Picqer refuses an empty one and takes a long one.
    TaskTitle, "task title", 1, usize::MAX
}

const TIME: &[FormatItem<'static>] = format_description!("[hour]:[minute]:[second]");

/// A time of the day as `HH:MM:SS`, such as `23:59:00`.
///
/// Picqer refuses `HH:MM` as a deadline, although its message asks for that.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TaskTime(time::Time);

impl TaskTime {
    /// Wraps a time of the day.
    pub const fn new(value: time::Time) -> Self {
        Self(value)
    }

    /// The wrapped value.
    pub const fn get(self) -> time::Time {
        self.0
    }
}

impl fmt::Display for TaskTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = self.0.format(TIME).map_err(|_| fmt::Error)?;
        f.write_str(&text)
    }
}

impl FromStr for TaskTime {
    type Err = ValidationError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        time::Time::parse(s, TIME)
            .map(Self)
            .map_err(|_| ValidationError::new("task time", format!("`{s}` is not `HH:MM:SS`")))
    }
}

impl Serialize for TaskTime {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for TaskTime {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        text.parse().map_err(serde::de::Error::custom)
    }
}

/// A task.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct Task {
    /// The task.
    pub idtask: TaskId,
    /// The title.
    pub title: String,
    /// The description.
    pub description: Option<String>,
    /// The user the task is assigned to.
    pub assigned_to_iduser: Option<UserId>,
    /// Whether the assignee has seen the task. `None` for a task that is not assigned.
    pub is_seen_by_assignee: Option<bool>,
    /// The deadline date, as a date and time at midnight.
    pub deadline_date: Option<DateTime>,
    /// The deadline time.
    pub deadline_time: Option<TaskTime>,
    /// When the task was completed.
    pub completed_at: Option<DateTime>,
    /// The user that completed the task.
    pub completed_by_iduser: Option<UserId>,
    /// The user that made the task.
    pub created_by_iduser: Option<UserId>,
    /// When the task was made.
    pub created_at: DateTime,
    /// When the task was last changed.
    pub updated_at: DateTime,
}

/// Filters for listing tasks.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TaskQuery {
    completed: Option<bool>,
}

impl TaskQuery {
    /// No filters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Lists the tasks that are completed, or those that are not.
    pub fn completed(mut self, completed: bool) -> Self {
        self.completed = Some(completed);
        self
    }

    pub(crate) fn pairs(self) -> Vec<(&'static str, String)> {
        self.completed
            .map(|completed| vec![("completed", completed.to_string())])
            .unwrap_or_default()
    }
}

/// A task to create.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NewTask {
    title: TaskTitle,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    deadline_date: Option<Date>,
    #[serde(skip_serializing_if = "Option::is_none")]
    deadline_time: Option<TaskTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    assigned_to_iduser: Option<UserId>,
}

impl NewTask {
    /// A task with a title.
    pub fn new(title: TaskTitle) -> Self {
        Self {
            title,
            description: None,
            deadline_date: None,
            deadline_time: None,
            assigned_to_iduser: None,
        }
    }

    /// Sets the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the deadline date.
    pub fn deadline_date(mut self, date: Date) -> Self {
        self.deadline_date = Some(date);
        self
    }

    /// Sets the deadline date and the time of the day.
    pub fn deadline(mut self, date: Date, time: TaskTime) -> Self {
        self.deadline_date = Some(date);
        self.deadline_time = Some(time);
        self
    }

    /// Assigns the task to a user.
    pub fn assigned_to(mut self, user: UserId) -> Self {
        self.assigned_to_iduser = Some(user);
        self
    }
}

/// Changes to a task. Only what is set is sent, except for the assignee.
///
/// Picqer takes the assignee away from a task whenever a change does not name one, so
/// [`Tasks::update`] sends the current assignee along when none is set here, which costs a call.
/// A deadline date cannot be taken away: Picqer makes it today when it is emptied, so only the
/// deadline time can be removed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TaskChanges {
    fields: Map<String, Value>,
    assignee: Option<Option<UserId>>,
}

impl TaskChanges {
    /// No changes.
    pub fn new() -> Self {
        Self::default()
    }

    /// Changes the title.
    pub fn title(mut self, title: TaskTitle) -> Self {
        self.fields.insert("title".into(), json!(title));
        self
    }

    /// Changes the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.fields
            .insert("description".into(), json!(description.into()));
        self
    }

    /// Removes the description.
    pub fn without_description(mut self) -> Self {
        self.fields.insert("description".into(), Value::Null);
        self
    }

    /// Changes the deadline date and keeps the deadline time.
    pub fn deadline_date(mut self, date: Date) -> Self {
        self.fields.insert("deadline_date".into(), json!(date));
        self
    }

    /// Changes the deadline date and the time of the day.
    pub fn deadline(mut self, date: Date, time: TaskTime) -> Self {
        self.fields.insert("deadline_date".into(), json!(date));
        self.fields.insert("deadline_time".into(), json!(time));
        self
    }

    /// Removes the deadline time and keeps the deadline date.
    pub fn without_deadline_time(mut self) -> Self {
        self.fields.insert("deadline_time".into(), Value::Null);
        self
    }

    /// Assigns the task to a user.
    pub fn assigned_to(mut self, user: UserId) -> Self {
        self.assignee = Some(Some(user));
        self
    }

    /// Takes the assignee away.
    pub fn unassigned(mut self) -> Self {
        self.assignee = Some(None);
        self
    }

    pub(crate) fn assignee(&self) -> Option<Option<UserId>> {
        self.assignee
    }

    pub(crate) fn to_value(&self, assignee: Option<UserId>) -> Value {
        let mut body = self.fields.clone();
        body.insert("assigned_to_iduser".into(), json!(assignee));
        Value::Object(body)
    }
}

pub(crate) fn list(query: TaskQuery, page: Page) -> Call<Vec<Task>> {
    let mut request = Request::new(Method::GET, segments(&[&"tasks"]));
    for (name, value) in query.pairs() {
        request = request.query(name, value);
    }
    Call::new(page.apply(request))
}

pub(crate) fn get(id: TaskId) -> Call<Task> {
    Call::new(Request::new(Method::GET, segments(&[&"tasks", &id])))
}

pub(crate) fn create(task: &NewTask) -> Result<Call<Task>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"tasks"])).json_body(task)?,
    ))
}

pub(crate) fn update(
    id: TaskId,
    changes: &TaskChanges,
    assignee: Option<UserId>,
) -> Result<Call<Task>> {
    Ok(Call::new(
        Request::new(Method::PUT, segments(&[&"tasks", &id]))
            .json_body(&changes.to_value(assignee))?,
    ))
}

pub(crate) fn delete(id: TaskId) -> NoContent {
    NoContent::new(Request::new(Method::DELETE, segments(&[&"tasks", &id])))
}

pub(crate) fn transition(id: TaskId, action: &'static str) -> Call<Task> {
    Call::new(Request::new(
        Method::POST,
        segments(&[&"tasks", &id, &action]),
    ))
}

macro_rules! tasks_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of tasks.
        pub $($async)? fn list(
            &self,
            query: $crate::tasks::TaskQuery,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::tasks::Task>> {
            self.client.execute($crate::tasks::list(query, page)) $(.$await)?
        }

        /// Lists all tasks that match a query, following the pages.
        pub $($async)? fn list_all(
            &self,
            query: $crate::tasks::TaskQuery,
        ) -> $crate::error::Result<Vec<$crate::tasks::Task>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::tasks::list(query, page)) $(.$await)?)
        }

        /// Fetches a task.
        pub $($async)? fn get(
            &self,
            id: $crate::types::TaskId,
        ) -> $crate::error::Result<$crate::tasks::Task> {
            self.client.execute($crate::tasks::get(id)) $(.$await)?
        }

        /// Creates a task.
        pub $($async)? fn create(
            &self,
            task: &$crate::tasks::NewTask,
        ) -> $crate::error::Result<$crate::tasks::Task> {
            self.client.execute($crate::tasks::create(task)?) $(.$await)?
        }

        /// Changes a task and returns it.
        ///
        /// Picqer takes the assignee away from a task whenever a change does not name one. When the
        /// changes do not set the assignee, the task is fetched first and its assignee is sent
        /// along, so it stays.
        pub $($async)? fn update(
            &self,
            id: $crate::types::TaskId,
            changes: &$crate::tasks::TaskChanges,
        ) -> $crate::error::Result<$crate::tasks::Task> {
            let assignee = match changes.assignee() {
                Some(assignee) => assignee,
                None => self.client.execute($crate::tasks::get(id)) $(.$await)? ?.assigned_to_iduser,
            };
            self.client.execute($crate::tasks::update(id, changes, assignee)?) $(.$await)?
        }

        /// Deletes a task.
        pub $($async)? fn delete(&self, id: $crate::types::TaskId) -> $crate::error::Result<()> {
            self.client.execute($crate::tasks::delete(id)) $(.$await)?
        }

        /// Marks a task as completed by the user of the API key. Completing one that is
        /// completed already changes nothing.
        pub $($async)? fn complete(&self, id: $crate::types::TaskId) -> $crate::error::Result<$crate::tasks::Task> {
            self.client.execute($crate::tasks::transition(id, "complete")) $(.$await)?
        }

        /// Marks a task as not completed.
        pub $($async)? fn uncomplete(&self, id: $crate::types::TaskId) -> $crate::error::Result<$crate::tasks::Task> {
            self.client.execute($crate::tasks::transition(id, "uncomplete")) $(.$await)?
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use tasks_methods;

/// Operations on tasks.
#[derive(Debug, Clone, Copy)]
pub struct Tasks<'a> {
    pub(crate) client: &'a Client,
}

impl Tasks<'_> {
    tasks_methods!(async; await);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_time_has_seconds_like_picqer_wants_it() {
        let time: TaskTime = "23:59:00".parse().unwrap();
        assert_eq!(time.to_string(), "23:59:00");
        assert!("23:59".parse::<TaskTime>().is_err());
        assert!("24:00:00".parse::<TaskTime>().is_err());
        assert_eq!(serde_json::to_value(time).unwrap(), json!("23:59:00"));
    }

    #[test]
    fn a_task_sends_what_is_set() {
        assert_eq!(
            serde_json::to_value(NewTask::new(TaskTitle::new("Restock").unwrap())).unwrap(),
            json!({"title": "Restock"})
        );
        let task = NewTask::new(TaskTitle::new("Restock").unwrap())
            .description("fifty units")
            .deadline("2031-03-04".parse().unwrap(), "23:59:00".parse().unwrap())
            .assigned_to(UserId(2));
        assert_eq!(
            serde_json::to_value(task).unwrap(),
            json!({
                "title": "Restock", "description": "fifty units", "deadline_date": "2031-03-04",
                "deadline_time": "23:59:00", "assigned_to_iduser": 2
            })
        );
        assert!(TaskTitle::new("").is_err());
    }

    #[test]
    fn changes_always_carry_an_assignee() {
        let changes = TaskChanges::new()
            .without_description()
            .without_deadline_time();
        assert_eq!(changes.assignee(), None);
        assert_eq!(
            changes.to_value(Some(UserId(3))),
            json!({"description": null, "deadline_time": null, "assigned_to_iduser": 3})
        );
        let unassigned = TaskChanges::new().unassigned();
        assert_eq!(unassigned.assignee(), Some(None));
        assert_eq!(
            unassigned.to_value(None),
            json!({"assigned_to_iduser": null})
        );
        assert_eq!(
            TaskChanges::new().assigned_to(UserId(4)).assignee(),
            Some(Some(UserId(4)))
        );
    }

    #[test]
    fn the_filter_becomes_a_query_pair() {
        assert!(TaskQuery::new().pairs().is_empty());
        assert_eq!(
            TaskQuery::new().completed(false).pairs(),
            vec![("completed", "false".to_owned())]
        );
    }
}
