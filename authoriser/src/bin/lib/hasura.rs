use jsonwebtoken::{encode, EncodingKey, Header};
use log::info;
use reqwest::blocking::Client;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

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
pub enum GetErrors {
    RequestFailed(String),
    NoUsersFound(String),
    TooManyUsersFound(String),
}

fn to_get_req_failed<Ok, Err: ToString>(err: Err) -> Result<Ok, GetErrors> {
    Err(GetErrors::RequestFailed(err.to_string()))
}

pub fn get_user_id(
    client: &Client,
    hasura_url: &str,
    facebookId: &str,
    admin_secret: &str,
) -> Result<String, GetErrors> {
    let mut variables = HashMap::new();
    variables.insert("facebook_id".to_owned(), facebookId.to_owned());

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
        .or_else(to_get_req_failed)?;

    let raw_body = response.text().or_else(to_get_req_failed)?;
    info!("received response from hasura get user fn: {}", raw_body);

    let body: UsersDataBody = serde_json::from_str(&raw_body).or_else(to_get_req_failed)?;

    match body.data.users.as_slice() {
        [user] => Ok(user.id.to_owned()),
        [] => Err(GetErrors::NoUsersFound(format!(
            "no users found in: {}",
            raw_body
        ))),
        _ => Err(GetErrors::TooManyUsersFound(format!(
            "too many users found in : {}",
            raw_body
        ))),
    }
}

#[derive(Deserialize, Debug)]
struct CreateUserResponseData {
    #[serde(rename = "insert_users_one")]
    user: User,
}

#[derive(Deserialize, Debug)]
struct CreateUserResponse {
    data: CreateUserResponseData,
}

#[derive(Debug)]
pub enum CreateErrors {
    RequestFailed(String),
    UsersExists(String),
}

fn to_create_req_failed<Ok, Err: ToString>(err: Err) -> Result<Ok, CreateErrors> {
    Err(CreateErrors::RequestFailed(err.to_string()))
}

pub fn create_user(
    client: &Client,
    hasura_url: &str,
    facebookId: &str,
    admin_secret: &str,
) -> Result<String, CreateErrors> {
    let mut variables = HashMap::new();
    variables.insert("facebook_id".to_owned(), facebookId.to_owned());
    let raw_body = serde_json::to_string(&GraphQLBody{
        operationName: "CreateUser".to_owned(),
        query: "mutation CreateUser($facebook_id: String) { insert_users_one(object: {facebook_id: $facebook_id}) { id } } ".to_owned(),
        variables: variables,
    }).expect("can't serialise body for hasura");
    let response = client
        .post(hasura_url)
        .body(raw_body)
        .header("x-hasura-admin-secret", admin_secret)
        .send()
        .or_else(to_create_req_failed)?;
    let raw_body = response.text().or_else(to_create_req_failed)?;
    info!("received response from hasura get user fn: {}", raw_body);
    let body: CreateUserResponse = serde_json::from_str(&raw_body).or_else(to_create_req_failed)?;
    Ok(body.data.user.id)
}

#[derive(Debug, Serialize, Deserialize)]
struct HasuraClaims {
    #[serde(rename = "x-hasura-default-role")]
    default_role: String,
    #[serde(rename = "x-hasura-allowed-roles")]
    allowed_roles: Vec<String>,
    #[serde(rename = "x-hasura-user-id")]
    user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    #[serde(rename = "https://hasura.io/jwt/claims")]
    hasura_claims: HasuraClaims,
    iat: u64,
    //   aud: 1516239022,
    exp: u64,
}

pub fn create_jwt(jwt_secret: String, user_id: String) -> Result<String, String> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .or_else(|err| Err(format!("{}", err)))?;
    let allowed_roles = vec!["user".to_owned()];

    let my_claims = Claims {
        iat: now.as_secs(),
        exp: (now + Duration::from_secs(12 * 24 * 60 * 60)).as_secs(),
        hasura_claims: HasuraClaims {
            default_role: "user".to_owned(),
            allowed_roles: allowed_roles,
            user_id: user_id,
        },
    };

    encode(
        &Header::default(),
        &my_claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
    .or_else(|err| Err(format!("{}", err)))
}
