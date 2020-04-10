use serde_derive::{Deserialize, Serialize};
use url::form_urlencoded;

#[derive(Deserialize, Debug)]
struct DebugTokenData {
    app_id: String,
    #[serde(rename = "type")]
    type_: String,
    application: String,
    data_access_expires_at: u64,
    expires_at: u64,
    is_valid: bool,
    scopes: Vec<String>,
    // granular_scopes:[
    //   {
    //     scope:manage_pages,
    //     target_ids:[
    //       {page-1-app-can-access-id},
    //       {page-2-app-can-access-id}
    //     ]
    //   },
    //   {
    //     scope:pages_show_list,
    //     target_ids:[
    //       {page-1-app-can-access-id},
    //       {page-2-app-can-access-id}
    //     ]
    //   }
    // ],
    user_id: String,
}

#[derive(Deserialize, Debug)]
struct DebugTokenResponse {
    data: DebugTokenData,
}

pub fn get_fb_id(admin_access_token: &str, user_access_token: &str) -> Result<String, String> {
    let req_url: String =
        form_urlencoded::Serializer::new("https://graph.facebook.com/debug_token".to_owned())
            .append_pair("input_token", user_access_token)
            .append_pair("access_token", admin_access_token)
            .finish();
    let result =
        reqwest::blocking::get(&req_url).and_then(|data| data.json::<DebugTokenResponse>());

    match result {
        Ok(resp) => {
            if resp.data.is_valid {
                Ok(resp.data.user_id)
            } else {
                Err("invalid access token".to_owned())
            }
        }
        Err(err) => Err(format!("failed to get result: {}", err)),
    }
}
