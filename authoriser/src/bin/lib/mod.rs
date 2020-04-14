mod fb;
mod hasura;

use lambda_http::{lambda, IntoResponse, Request, Response};
use lambda_runtime::{error::HandlerError, Context};
use log::{error, info};
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
struct HasuraBody<Input> {
    // action: String,
    input: Input,
    // session_variables: kv pair,
}

#[derive(Deserialize, Debug)]
struct VerifyFBAccessInput {
    #[serde(rename = "fbToken")]
    fb_access_token: String,
}

#[derive(Serialize, Debug)]
struct VerifyFBAccessOutput {
    ok: bool,
    #[serde(rename = "accessToken")]
    access_token: String,
}

#[derive(Serialize, Debug)]
struct HasuraError<'a> {
    message: &'a String,
    code: Option<&'static str>,
}

fn handle(serde_result: Result<HasuraBody<VerifyFBAccessInput>, String>) -> (u16, String) {
    let admin_access_token = std::env::var("FB_ACCESS_TOKEN").expect("FB_ACCESS_TOKEN not defined");
    let jwt_key = std::env::var("JWT_KEY").expect("JWT_KEY not defined");
    let hasura_url = std::env::var("HASURA_API_URL").expect("HASURA_API_URL not defined");
    let hasura_admin_secret =
        std::env::var("HASURA_ADMIN_SECRET").expect("HASURA_ADMIN_SECRET not defined");

    let client = reqwest::blocking::Client::new();

    let res = serde_result
        .and_then(|body| {
            //grab access token
            let access_token = &body.input.fb_access_token;

            // verify with facebook
            fb::get_fb_id(&admin_access_token, access_token)
        })
        .and_then(|fb_user_id| {
            // get hasura id
            let res = hasura::get_user_id(&client, &hasura_url, &fb_user_id, &hasura_admin_secret);

            let user_id = match res {
                Ok(user_id) => Ok(user_id),
                Err(hasura::GetErrors::RequestFailed(msg)) => {
                    Err(format!("failed to get user from hasura: {}", msg))
                }
                Err(hasura::GetErrors::TooManyUsersFound(msg)) => {
                    error!("too many users found for id: {}", msg);
                    Err("Something went wrong".to_owned())
                }
                Err(hasura::GetErrors::NoUsersFound(_)) => {
                    let res = hasura::create_user(
                        &client,
                        &hasura_url,
                        &fb_user_id,
                        &hasura_admin_secret,
                    );

                    res.or_else(|err| {
                        Err(match err {
                            hasura::CreateErrors::RequestFailed(msg) => msg,
                            hasura::CreateErrors::UsersExists(msg) => msg,
                        })
                    })
                    .or_else(|msg| Err(format!("failed to create user user from hasura: {}", msg)))
                }
            }?;

            let jwt = hasura::create_jwt(jwt_key, user_id)?;

            serde_json::to_string(&VerifyFBAccessOutput {
                ok: true,
                access_token: jwt,
            })
            .or_else(|err| Err(format!("{}", err)))
        });

    match res {
        Ok(res) => (200, res),
        Err(msg) => {
            let decode_msg = format!("could not decode body: {}", msg);
            error!("{}", decode_msg);

            (
                400,
                match serde_json::to_string(&HasuraError {
                    message: &decode_msg,
                    code: None,
                }) {
                    Ok(body) => body,
                    Err(e) => format!("could not encode response: {}, in {}", e, decode_msg),
                },
            )
        }
    }
}

pub fn handler(req: Request) -> Response<String> {
    let raw_body = req.into_body();
    info!(
        "received request with body: {}",
        String::from_utf8_lossy(&raw_body.to_vec())
    );

    let res = serde_json::from_slice::<HasuraBody<VerifyFBAccessInput>>(&raw_body)
        .or_else(|err| Err(format!("{}", err)));

    let (code, body) = handle(res);

    Response::builder()
        .status(code)
        .body(body)
        .expect("something went terribly wrong")
}
