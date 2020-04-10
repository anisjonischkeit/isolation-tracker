mod fb;
mod hasura;

use lambda_http::{lambda, IntoResponse, Request, Response};
use lambda_runtime::{error::HandlerError, Context};
use log::{self, error, info};
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
    let admin_access_token = std::env::var("FB_ACCESS_TOKEN").unwrap();
    let jwt_key = std::env::var("JWT_KEY").unwrap();

    let res = serde_result
        .and_then(|body| {
            //grab access token
            let access_token = &body.input.fb_access_token;

            // verify with facebook
            fb::get_fb_id(&admin_access_token, access_token)
        })
        .and_then(|user_id| {
            // get hasura id
            hasura::get_or_create_user(&user_id);

            // create hasura jwt

            // return token
            serde_json::to_string(&VerifyFBAccessOutput {
                ok: true,
                access_token: user_id,
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
