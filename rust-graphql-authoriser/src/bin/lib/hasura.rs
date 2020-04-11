use log::info;
use reqwest::blocking::Client;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
struct User {
    id: String,
}

#[derive(Deserialize, Debug)]
struct UsersData {
    users: Vec<User>,
}

#[derive(Deserialize, Debug)]
struct UsersDataBody {
    data: UsersData,
}

#[derive(Serialize, Debug)]
struct GraphQLBody {
    operationName: String,
    query: String,
    variables: HashMap<String, String>,
}

#[derive(Debug)]
pub enum Errors {
    HasuraRequestFailed(String),
}

fn to_req_failed<Ok, Err: ToString>(err: Err) -> Result<Ok, Errors> {
    Err(Errors::HasuraRequestFailed(err.to_string()))
}

pub fn get_user_id(
    client: &Client,
    hasura_url: &str,
    facebookId: &str,
    admin_secret: &str,
) -> Result<String, Errors> {
    let mut variables = HashMap::new();
    variables.insert("$facebook_id".to_owned(), facebookId.to_owned());

    let raw_body = serde_json::to_string(&GraphQLBody{
        operationName: "GetUserId".to_owned(),
        query: "query GetUserId ($facebook_id: String!) { users(where: {facebook_id: {_eq: $facebook_id}}) { id }}".to_owned(),
        variables: variables,
    }).expect("can't serialise body for hasura");

    let response = client
        .post(hasura_url)
        .body(raw_body)
        .header("x-hasura-admin-secret", admin_secret)
        .send()
        .or_else(to_req_failed)?;

    let raw_body = response.text().or_else(to_req_failed)?;
    info!("received response from hasura get user fn: {}", raw_body);

    let body: UsersDataBody = serde_json::from_str(&raw_body).or_else(to_req_failed)?;

    match body.data.users.as_slice() {
        [user] => Ok(user.id.to_owned()),
        _ => to_req_failed(format!("wrong amount of users returned in: {}", raw_body)),
    }
}
