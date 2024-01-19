use axum::{debug_handler, extract::Query, http::StatusCode, response::IntoResponse, Json};
use reqwest;
use serde_derive::Deserialize;
use serde_json::json;

use crate::models::{AddFilterRequest, DataStore, Line, ItemLine, QueryResponse, ItemQueryResponse, ApiResponse, self};
use crate::redis::RedisInstance;

// Structs
#[derive(Deserialize, Debug)]
struct Config {
    ninja: Ninja,
}
#[derive(Deserialize, Debug)]
struct Ninja {
    currency: String,
    item: String,
}
#[derive(Deserialize, Debug)]
pub struct QueryParams {
    league: String,
    category: String,
}

impl QueryParams {
    pub fn new(league: String, category: String) -> QueryParams {
        QueryParams { league, category }
    }
}

// Public functions
// Heartbeat function
pub async fn hb() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}

// Get data from ninja
pub async fn get_data_from_ninja(
    query_params: Query<QueryParams>,
) -> (StatusCode, Json<ApiResponse>) {
    // Trace the request
    tracing::info!(
        "Requesting data from ninja: {} {}",
        query_params.league,
        query_params.category
    );
    // Build the request
    let api_response = request_data_from_ninja(
        query_params.league.as_str(),
        query_params.category.as_str(),
    ).await;
    // 根據 trait 類型
    let main_category = get_query_type(query_params.category.to_string());
    match main_category.as_str() {
        "Currency" => {
            if let Some(currency_response) = &api_response.currency_response {
                write_to_redis(query_params.league.as_str(), &main_category, &query_params.category.to_string(), currency_response);
            }
            return (StatusCode::OK, Json(api_response));
        }
        _ => {
            if let Some(item_response) = &api_response.item_response {
                write_to_redis_item(query_params.league.as_str(), main_category.as_str(), query_params.category.as_str(), item_response);
            }
            return (StatusCode::OK, Json(api_response));
        }
    } 
}

// Add a dataFilter and return the filterList
pub async fn add_data_filter(Json(payload): Json<AddFilterRequest>) -> (StatusCode, &'static str) {
    let mut redis_instance = get_redis_instance();
    let filter_type = payload.filter_type;
    let main_category = get_query_type(filter_type.to_string());
    let name = payload.name;
    let redis_key = format!("{}:{}:filter", main_category, filter_type);
    let existance = redis_instance.exist_in_list(redis_key.as_str(), name.as_str());
    // if exist skip else add
    if existance.is_ok() && existance.unwrap() == true {
        tracing::debug!("Filter {} already exists", redis_key);
        (StatusCode::OK, "Already exists")
    } else {
        let res = redis_instance.push_list(&redis_key, &name);
        match res {
            Ok(_) => {
                tracing::debug!("Filter {} {} added", redis_key, name);
                (StatusCode::OK, "Ok!")
            }
            Err(e) => {
                tracing::error!("Add filter error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Add filter failed!")
            }
        }
    }
}
// 取得 Redis 中的 filterList
pub async fn get_filter_data(
    query_params: Query<QueryParams>,
) -> (StatusCode, Json<Vec<DataStore>>) {
    let data_list = get_current_data_list(query_params.category.as_str());
    if data_list.len() != 0 {
        return (StatusCode::OK, Json(data_list));
    }
    else {
        tracing::info!("No data in Redis");
        return (StatusCode::OK, Json(Vec::new()));
    }
}
// Add a skip check 
pub async fn add_skip_check(Json(payload): Json<AddFilterRequest>) -> (StatusCode, &'static str) {
    let mut redis_instance = get_redis_instance();
    // check if exists in list
    let skip_key = format!("{}:skip", payload.filter_type);
    let existance = redis_instance.exist_in_list(skip_key.as_str(), payload.name.as_str());
    // if exist skip else add
    if existance.is_ok() && existance.unwrap() == true {
        tracing::debug!("Skip check {} already exists", skip_key);
        (StatusCode::OK, "Already exists")
    } else {
        let res = redis_instance.push_list(&skip_key, &payload.name);
        match res {
            Ok(_) => {
                tracing::debug!("Skip check {} {} added", skip_key, payload.name);
                (StatusCode::OK, "Ok!")
            }
            Err(e) => {
                tracing::debug!("Add skip check error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Add skip check failed!")
            }
        }
    }
}
// Get the filterList and format to the output format
pub fn get_format_output() -> String {
    let currency_list = get_all_category("Currency:*");
    let item_list = get_all_category("Item:*");
    // Format the output to Discord
    let mut output = String::new();
    // Currency header
    output.push_str("# Currency\n");
    // Build the currency table
    for c in currency_list.iter() {
        build_output_str(&mut output, &c);
    }
    // Item header
    output.push_str("# Item\n");
    for i in item_list.iter() {
        build_output_str(&mut output, &i);
    }
    output
}

// get data from ninja
pub async fn request_data_from_ninja(league: &str, category: &str) -> ApiResponse{
    // Build the request
    let client = reqwest::Client::new();
    let main_category = get_query_type(category.to_string());
    let url = format!(
        "{}?league={}&type={}",
        get_base_url_from_toml(category.to_string()),
        league,
        category
    );
    tracing::debug!("URL: {}", url);
    let res = client.get(&url).send().await;
    match res {
        Ok(res) => {
            let content = res.text().await;
            if main_category == "Currency" {
                let res_json: Result<QueryResponse, _> = serde_json::from_str(&content.unwrap());
                match res_json {
                    Ok(res_json) => {
                        let mut api_response = ApiResponse::empty();
                        api_response.set_currency_response(res_json);
                        return api_response;
                    }
                    Err(e) => {
                        tracing::error!("Parse data error: {}", e);
                        return ApiResponse::empty();
                    }
                }
            }
            else {
                let res_json: Result<ItemQueryResponse, _> = serde_json::from_str(&content.unwrap());
                match res_json {
                    Ok(res_json) => {
                        let mut api_response = ApiResponse::empty();
                        api_response.set_item_response(res_json);
                        return api_response;
                    }
                    Err(e) => {
                        tracing::error!("Parse data error: {}", e);
                        return ApiResponse::empty();
                    }
                }
            }
        }
        Err(e) => {
            tracing::error!("Get data error: {}", e);
            return ApiResponse::empty();
        }
    }
}

// Private functions
fn get_query_type(category: String) -> String {
    match category.as_str() {
        "Currency" => "Currency",
        "Fragment" => "Currency",
        _ => "Item",
    }
    .to_string()
}

fn get_base_url_from_toml(category: String) -> String {
    let config_value: String =
        std::fs::read_to_string("Config.toml").expect("Unable to read config file");
    let config: Config = toml::from_str(&config_value).unwrap();
    tracing::debug!("Config: {:?}", config);
    match category.as_str() {
        "Currency" => config.ninja.currency,
        "Fragment" => config.ninja.currency,
        _ => config.ninja.item,
    }
}

// Store the query data to redis
fn write_to_redis(league: &str, main_category: &str, category: &str, query_res: &QueryResponse) {
    // Initialize Redis
    let mut redis_instance = get_redis_instance();
    let mut data_list: Vec<String> = Vec::new();
    let mut current_list = get_current_data_list(category);
    let query_res = query_res.clone();
    for line in query_res.lines {
        // Match the currency
        let redis_key = format!("{}:{}:filter", main_category, category);
        // Check if the key exists
        let existance = redis_instance.exist_in_list(&redis_key, &line.currencyTypeName);
        let skip_key = format!("{}:skip", main_category);
        // Check if the category is in skip list, if yes skip filter
        let skip_filter = redis_instance.exist_in_list(&skip_key , &category);
        // parse to DataStore
        let data = parse_data_line_to_datastore(league, line, &redis_key, &mut redis_instance);
        // if exist update else skip
        match existance {
            Ok(existance) => {
                if existance == true {
                    tracing::debug!("{} {} exists", category, redis_key);
                    // If exist in current list skip
                    if exist_in_list(&current_list, &data.name) {
                        tracing::debug!("{} {} exists in current list pop first", category, &data.name);
                        pop_by_name(&mut current_list, &data.name);
                    }
                    data_list.push(data.to_json_string());
                } else {
                    // Check if the currency is in skip list, if yes skip filter
                    match skip_filter {
                        Ok(skip_filter) => {
                            if skip_filter == true {
                                tracing::debug!("{} exists in skip filter list", category);
                                // If exist in current list update it
                                if exist_in_list(&current_list, &data.name) {
                                    tracing::debug!("{} {} exists in current list pop first", category, &data.name);
                                    pop_by_name(&mut current_list, &data.name);
                                }
                                data_list.push(data.to_json_string());
                            }
                        }
                        Err(e) => {
                            tracing::error!("Redis execution error: {}", e);
                            continue;
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("Redis execution error: {}", e);
                continue;
            }
        }
    }
    if data_list.len() == 0 {
        tracing::debug!("No data to write");
        return;
    }
    // Write to Redis
    let redis_key = format!("{}:{}", main_category, category);
    // Remove all first
    let _ = redis_instance.delete(&redis_key);
    let res = redis_instance.set_list_expire(redis_key.as_str(), &data_list, 3600);
    match res {
        Ok(_) => {
            tracing::debug!("{} data written to Redis", category);
        }
        Err(e) => {
            tracing::error!("Write to Redis error: {}", e);
        }
    }
}

fn parse_data_line_to_datastore(league: &str, line: Line, redis_key: &str, redis_instance: &mut RedisInstance) -> DataStore {
    // if sparkline exists get the totalChange or 0
    let pay_total_change = match line.paySparkLine {
        Some(pay_spark_line) => pay_spark_line.totalChange,
        None => {
            tracing::debug!("{} paySparkLine not exists", redis_key);
            0.0
        }
    };
    let receive_total_change = match line.receiveSparkLine {
        Some(receive_spark_line) => receive_spark_line.totalChange,
        None => {
            tracing::debug!("{} receiveSparkLine not exists", redis_key);
            0.0
        }
    };
    let mut divine_equivalent = get_divine_to_chaos_ratio(league, redis_instance);
    divine_equivalent = line.chaosEquivalent / divine_equivalent;
    // Build the data
    let data = DataStore::new(
        line.currencyTypeName,
        line.chaosEquivalent,
        divine_equivalent,
        pay_total_change,
        receive_total_change,
        chrono::offset::Utc::now().to_string(),
    );
    if data.name == "Divine Orb" {
        // delete first 
        let _ = redis_instance.delete("Currency:D2C");
        let _ = redis_instance.push_hash_expire(&format!("Currency:D2C"),"ratio", &data.chaos_equivalent.to_string(), 3600);
        let _ = redis_instance.push_hash_expire(&format!("Currency:D2C"),"update_time", &chrono::offset::Utc::now().to_string(), 3600);
    }
    data
}

// write to Redis item flow
fn write_to_redis_item(league: &str, main_category: &str, category: &str, query_res: &ItemQueryResponse) {
    let mut redis_instance = get_redis_instance();
    let mut data_list: Vec<String> = Vec::new();
    let mut current_list = get_current_data_list(category);
    let query_res = query_res.clone();
    for line in query_res.lines {
        // Match the currency
        let redis_key = format!("{}:{}:filter", main_category, category);
        // Check if the key exists
        let existance = redis_instance.exist_in_list(&redis_key, &line.name);
        let skip_key = format!("{}:skip", main_category);
        // Check if the category is in skip list, if yes skip filter
        let skip_filter = redis_instance.exist_in_list(&skip_key , &category);
        // parse to DataStore
        let data = parse_data_line_to_datastore_item(league, line, &redis_key, &mut redis_instance);
        // if exist update else skip
        match existance {
            Ok(existance) => {
                if existance == true {
                    tracing::info!("{} {} exists", category, redis_key);
                    // If exist in current list skip
                    if exist_in_list(&current_list, &data.name) {
                        tracing::info!("{} {} exists in current list pop first", category, &data.name);
                        pop_by_name(&mut current_list, &data.name);
                    }
                    data_list.push(data.to_json_string());
                } else {
                    // Check if the currency is in skip list, if yes skip filter
                    match skip_filter {
                        Ok(skip_filter) => {
                            if skip_filter == true {
                                tracing::info!("{} exists in skip filter list", category);
                                // If exist in current list update it
                                if exist_in_list(&current_list, &data.name) {
                                    tracing::info!("{} {} exists in current list pop first", category, &data.name);
                                    pop_by_name(&mut current_list, &data.name);
                                }
                                // TODO: Set in config
                                if data.chaos_equivalent > 10.0 {
                                    data_list.push(data.to_json_string());
                                }
                                else {
                                    tracing::debug!("{} priced {} chaos too cheap to skip", data.name, data.chaos_equivalent);
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Redis execution error: {}", e);
                            continue;
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("Redis execution error: {}", e);
                continue;
            }
        }
    }
    // Write to Redis
    let redis_key = format!("{}:{}", main_category, category);
    // Remove all first
    let _ = redis_instance.delete(&redis_key);
    let res = redis_instance.set_list_expire(redis_key.as_str(), &data_list, 3600);
    match res {
        Ok(_) => {
            tracing::info!("{} data written to Redis", category);
        }
        Err(e) => {
            tracing::error!("Write to Redis error: {}", e);
        }
    }
}

// parse item line to DataStore
fn parse_data_line_to_datastore_item(league: &str, line: ItemLine, redis_key: &str, redis_instance: &mut RedisInstance) -> DataStore {
    // if sparkline exists get the totalChange or 0
    let total_change = match line.sparkline {
        Some(spark_line) => spark_line.totalChange,
        None => {
            tracing::info!("{} sparkLine not exists", redis_key);
            0.0
        }
    };
    let mut divine_equivalent = get_divine_to_chaos_ratio(league, redis_instance);
    divine_equivalent = line.chaosValue / divine_equivalent;
    // Build the data
    let data = DataStore::new(
        line.name.clone(),
        line.chaosValue,
        divine_equivalent,
        total_change,
        total_change,
        chrono::offset::Utc::now().to_string(),
    );
    data
}

// Get Redis instance
fn get_redis_instance() -> RedisInstance {
    let redis_instance = RedisInstance::new();
    redis_instance
}
// Get current data list from Redis
fn get_current_data_list(category: &str) -> Vec<DataStore> {
    let mut redis_instance = get_redis_instance();
    let redis_key = format!(
        "{}:{}",
        get_query_type(category.to_string()),
        category
    );
    let res = redis_instance.get_list(redis_key.as_str());
    match res {
        Ok(_) => {
            // Parse to DataStore
            let mut data_list: Vec<DataStore> = Vec::new();
            for data in res.unwrap() {
                let data_json = serde_json::from_str::<DataStore>(&data);
                match data_json {
                    Ok(data_json) => {
                        data_list.push(data_json);
                    }
                    Err(e) => {
                        tracing::error!("Parse data error: {}", e);
                        return Vec::new();
                    }
                }
            }
            return data_list;
        }
        Err(e) => {
            tracing::error!("Get filter data error: {}", e);
            return Vec::new();
        }
    }
}

// Exist in list
fn exist_in_list(list: &Vec<DataStore>, value: &String) -> bool {
    for item in list.iter() {
        if item.name == value.to_string() {
            return true;
        }
    }
    false
}
// Pop by name
fn pop_by_name(list: &mut Vec<DataStore>, value: &String) {
    let mut index = 0;
    for item in list.iter() {
        if item.name == value.to_string() {
            break;
        }
        index += 1;
    }
    tracing::info!("Pop index: {}, value: {}", index, value);
    list.remove(index);
}

// Get all category from Redis
fn get_all_category(key: &str) -> Vec<String> {
    let mut redis_instance = get_redis_instance();
    let category_list = redis_instance.get_all_keys_name(key);
    match category_list {
        Ok(category_list) => {
            return category_list;
        }
        Err(e) => {
            tracing::error!("Get all category error: {}", e);
            return Vec::new();
        }
    }
}
// Build the output string
fn build_output_str(output: &mut String, category: &str) {
    let data_list = get_current_data_list(category);
    // Markdown header
    output.push_str(format!("## {}\n", category).as_str());
    output.push_str("| name | Chaos Equivalent | Pay Total Change | Receive Total Change |\n");
    output.push_str("| --- | --- | --- | --- |\n");
    for data in data_list.iter() {
        output.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            data.name,
            data.chaos_equivalent,
            data.pay_total_change,
            data.receive_total_change
        ));
    }
}
// Get divine to chaos ratio
fn get_divine_to_chaos_ratio(league: &str, redis: &mut RedisInstance) -> f64 {
    let divine_to_chaos = redis.get_hash(format!("{}:D2C", league).as_str(), "ratio");
    match divine_to_chaos {
        Ok(divine_to_chaos) => {
            let divine_to_chaos: f64 = divine_to_chaos.parse().unwrap();
            divine_to_chaos
        }
        Err(e) => {
            tracing::error!("Get divine to chaos ratio error: {}", e);
            return 0.0;
        }
    }
}

// Tests
#[cfg(test)]
#[tokio::test]
async fn test_get_data_from_ninja() {
    let league = String::from("Affliction");
    let category = String::from("Currency");
    let query_params = QueryParams { league, category };
    let (status, response) = get_data_from_ninja(Query(query_params)).await;
    assert_eq!(status, StatusCode::OK);
    let output = get_format_output();

    assert_ne!(output, "");
}
