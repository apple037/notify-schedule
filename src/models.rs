use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryResponse {
    pub lines:  Vec<Line>,
    pub currencyDetails: Vec<CurrencyDetails>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Line {
    pub currencyTypeName: String,
    pub chaosEquivalent: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CurrencyDetails {
    pub icon: String,
    pub name: String,
    pub tradeId: String,
}
impl QueryResponse {
    pub fn empty() -> QueryResponse {
        QueryResponse {
            lines: Vec::new(),
            currencyDetails: Vec::new(),
        }
    }
}
