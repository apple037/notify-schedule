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
pub struct DataStore {
    pub currency: String,
    pub chaos_equivalent: f64,
    pub pay_total_change: f64,
    pub receive_total_change: f64,
}

impl DataStore {
    pub fn new(
        currency: String,
        chaos_equivalent: f64,
        pay_total_change: f64,
        receive_total_change: f64,
    ) -> DataStore {
        DataStore {
            currency,
            chaos_equivalent,
            pay_total_change,
            receive_total_change,
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
