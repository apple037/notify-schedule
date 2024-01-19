use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryResponse {
    pub lines: Vec<Line>,
    // pub currencyDetails: Vec<CurrencyDetails>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Line {
    pub currencyTypeName: String,
    pub chaosEquivalent: f64,
    pub paySparkLine: Option<PaySparkLine>,
    pub receiveSparkLine: Option<ReceiveSparkLine>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CurrencyDetails {
    pub icon: String,
    pub name: String,
    pub tradeId: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PaySparkLine {
    // pub data: Vec<f64>,
    pub totalChange: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReceiveSparkLine {
    // pub data: Vec<f64>,
    pub totalChange: f64,
}

impl QueryResponse {
    pub fn empty() -> QueryResponse {
        QueryResponse {
            lines: Vec::new(),
            // currencyDetails: Vec::new(),
        }
    }
}

impl Clone for QueryResponse {
    fn clone(&self) -> QueryResponse {
        QueryResponse {
            lines: self.lines.clone(),
            // currencyDetails: self.currencyDetails.clone(),
        }
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ItemQueryResponse {
    pub lines: Vec<ItemLine>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemLine {
    pub name: String,
    pub chaosValue: f64,
    pub divineValue: f64,
    pub sparkline: Option<SparkLine>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SparkLine {
    pub totalChange: f64,
}

impl ItemQueryResponse {
    pub fn empty() -> ItemQueryResponse {
        ItemQueryResponse { lines: Vec::new() }
    }
}

impl Clone for ItemQueryResponse {
    fn clone(&self) -> ItemQueryResponse {
        ItemQueryResponse {
            lines: self.lines.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponse {
    pub item_response: Option<ItemQueryResponse>,
    pub currency_response: Option<QueryResponse>,
}

impl ApiResponse {
    pub fn empty() -> ApiResponse {
        ApiResponse {
            item_response: None,
            currency_response: None,
        }
    }

    pub fn set_item_response(&mut self, item_response: ItemQueryResponse) {
        self.item_response = Some(item_response);
    }

    pub fn set_currency_response(&mut self, currency_response: QueryResponse) {
        self.currency_response = Some(currency_response);
    }
}

impl Clone for ApiResponse {
    fn clone(&self) -> ApiResponse {
        ApiResponse {
            item_response: self.item_response.clone(),
            currency_response: self.currency_response.clone(),
        }
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct DataStore {
    pub name: String,
    pub chaos_equivalent: f64,
    pub divine_equivalent: f64,
    pub pay_total_change: f64,
    pub receive_total_change: f64,
    pub update_time: String,
}

impl DataStore {
    pub fn new(
        name: String,
        chaos_equivalent: f64,
        divine_equivalent: f64,
        pay_total_change: f64,
        receive_total_change: f64,
        update_time: String,
    ) -> DataStore {
        DataStore {
            name,
            chaos_equivalent,
            divine_equivalent,
            pay_total_change,
            receive_total_change,
            update_time,
        }
    }

    pub fn to_json_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddFilterRequest {
    pub filter_type: String,
    pub name: String,
}
