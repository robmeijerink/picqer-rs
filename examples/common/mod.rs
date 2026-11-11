use picqer_rs::{Client, UserAgent};

/// Builds a client from `PICQER_SUBDOMAIN` and `PICQER_API_KEY`, or from a `.env` file's values
/// when you export them first.
pub fn client() -> Client {
    let user_agent =
        UserAgent::new("picqer-rs examples", "dev@example.com").expect("the user agent is valid");
    Client::from_env(user_agent).expect("PICQER_SUBDOMAIN and PICQER_API_KEY must be set")
}
