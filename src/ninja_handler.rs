use axum::{
    http::StatusCode,
    response::IntoResponse,
    Json
};
use serde_derive::Deserialize;
use serde_json::json;
use reqwest;

// nonja_handler.rs
use crate::models::QueryResponse;


#[derive(Deserialize, Debug)]
struct Config {
    discord: Discord,
    ninja: Ninja
}
#[derive(Deserialize, Debug)]
struct Discord {
    api_key: String,
}
#[derive(Deserialize, Debug)]
struct Ninja {
    currency: String,
    item: String
}

fn get_base_url_from_toml(category: String) -> String {
    let config_value: String = std::fs::read_to_string("Config.toml").expect("Unable to read config file");
    let config: Config = toml::from_str(&config_value).unwrap();
    tracing::info!("Config: {:?}", config);
    let c = "currency".to_string();
    let i = "item".to_string();
    match category {
        c => config.ninja.currency,
        i => config.ninja.item,
        _ => panic!("Invalid category")
    }
}

pub async fn hb() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}

pub async fn get_data_from_ninja(league: String, category: String) -> (StatusCode, Json<QueryResponse>) {
    // Trace the request
    tracing::info!("Requesting data from ninja: {} {}", league, category);
    // Build the request
    let client = reqwest::Client::new();
    let url = format!("{}?league={}&type={}", get_base_url_from_toml(category.to_string()), league, category);
    tracing::info!("URL: {}", url);
    let res = client.get(&url)
        .send()
        .await;
    match res {
        Ok(res) => {
            // Parse to QueryResponse
            let res_json = res.json::<QueryResponse>().await;
            match res_json {
                Ok(res_json) => {
                    tracing::info!("Response: {:?}", res_json);
                    return (StatusCode::OK, Json(res_json));
                },
                Err(e) => {
                    tracing::error!("Parse data error: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(QueryResponse::empty()));
                }
            }
        },
        Err(e) => {
            tracing::error!("Get data error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(QueryResponse::empty()));
        }
    }
}


#[cfg(test)]
fn test_get_data_from_ninja() {
    let league = String::from("Affliction");
    let category = String::from("Currency");
    let res = get_data_from_ninja(league, category);
}