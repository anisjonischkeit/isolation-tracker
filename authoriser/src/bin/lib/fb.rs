use log::info;
use serde_derive::{Deserialize, Serialize};
use serde_json::from_str;
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
    let req_url: String = format!(
        "https://graph.facebook.com/debug_token?{}",
        form_urlencoded::Serializer::new(String::new())
            .append_pair("input_token", user_access_token)
            .append_pair("access_token", admin_access_token)
            .finish()
    );
    reqwest::blocking::get(&req_url)
        .or_else(|err| Err(format!("{}", err)))
        .and_then(|data| {
            if data.status().is_success() {
                data.text().or_else(|err| Err(format!("{}", err)))
            } else {
                Err("error while fetching debug_token".to_owned())
            }
        })
        .and_then(|text| {
            info!("fb debug token responded with: {}", text);
            serde_json::from_str::<DebugTokenResponse>(&text)
                .or_else(|err| Err(format!("failed to encode json: {}", err)))
        })
        .and_then(|resp| {
            if resp.data.is_valid {
                Ok(resp.data.user_id)
            } else {
                Err("invalid access token".to_owned())
            }
        })
        .or_else(|err| Err(format!("get_fb_id failed: {}", err)))
}
