use futures::future::TryFutureExt;
use snafu::futures::try_future::TryFutureExt as SnafuTryFutureExt;

use super::users::{MultiUsersResponseBody, SingleUserResponseBody, UserRequestBody};
use crate::error;
use crate::utils::{construct_headers, get_service_url};

pub async fn list_users() -> Result<MultiUsersResponseBody, error::Error> {
    let data = "{ \"query\": \"{ users { users { username, email }, usersCount } }\" }";
    let url = get_service_url();
    let client = reqwest::Client::new();
    client
        .post(&url)
        .headers(construct_headers())
        .body(data)
        .send()
        .context(error::ReqwestError {
            msg: String::from("Could not query users"),
        })
        .and_then(|resp| {
            resp.json::<serde_json::Value>()
                .context(error::ReqwestError {
                    msg: String::from("Could not deserialize MultiUsersResponseBody"),
                })
        })
        .and_then(|json| {
            async move {
                let res = &json["data"]["users"];
                let res = res.clone();
                serde_json::from_value(res)
            }
            .context(error::JSONError {
                msg: String::from("Could not deserialize MultiUsersResponseBody"),
            })
        })
        .await
}

pub async fn add_user(user: UserRequestBody) -> Result<SingleUserResponseBody, error::Error> {
    let query = r#" "mutation addUser($user: UserRequestBody!) { addUser(user: $user) { user { id, username, email, active, createdAt, updatedAt } } }"#;
    let variables = serde_json::to_string(&user).unwrap();
    let data = format!(
        r#"{{ "query": {query}, "variables": {variables} }}"#,
        query = query,
        variables = variables
    );
    let url = get_service_url();
    let client = reqwest::Client::new();
    client
        .post(&url)
        .headers(construct_headers())
        .body(data)
        .send()
        .context(error::ReqwestError {
            msg: String::from("Could not request SingleUserResponseBody"),
        })
        .and_then(|resp| {
            resp.json::<serde_json::Value>()
                .context(error::ReqwestError {
                    msg: String::from("Could not deserialize MultiUsersResponseBody"),
                })
        })
        .and_then(|json| {
            async move {
                let res = &json["data"]["addUser"];
                let res = res.clone();
                serde_json::from_value(res)
            }
            .context(error::JSONError {
                msg: String::from("Could not deserialize MultiUsersResponseBody"),
            })
        })
        .await
}

pub mod blocking {
    use crate::api::users::{MultiUsersResponseBody, SingleUserResponseBody, UserRequestBody};
    use crate::error;
    pub fn list_users() -> Result<MultiUsersResponseBody, error::Error> {
        // We use the Client API, which is async, so we need to wrap it around some
        // tokio machinery to spin the async code in a thread, and wait for the result.
        let handle = tokio::runtime::Handle::current();
        let th = std::thread::spawn(move || handle.block_on(async { super::list_users().await }));
        th.join().unwrap()
    }
    pub fn add_user(user: UserRequestBody) -> Result<SingleUserResponseBody, error::Error> {
        // We use the Client API, which is async, so we need to wrap it around some
        // tokio machinery to spin the async code in a thread, and wait for the result.
        let handle = tokio::runtime::Handle::current();
        let th = std::thread::spawn(move || handle.block_on(async { super::add_user(user).await }));
        th.join().unwrap()
    }
}