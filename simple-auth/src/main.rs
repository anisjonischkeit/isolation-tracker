use serde::Deserialize;
use warp::Filter;

#[derive(Debug, Deserialize)]
pub struct FBLoginCallbackQueryArgs {
    pub access_token: String,
}

#[tokio::main]
async fn main() {
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("fb-login-callback")
        .and(warp::get())
        .and(warp::query())
        .map(|query_args: FBLoginCallbackQueryArgs| format!("Hello, {}!", query_args.access_token));

    warp::serve(hello).run(([127, 0, 0, 1], 3030)).await;
}
