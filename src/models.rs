use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AccountQuery {
    #[serde(rename = "accountNo")]
    pub account_no: String,
}

#[derive(Deserialize, Serialize)]
pub struct AccountStatusRequest {
    #[serde(rename = "accountNo")]
    pub account_no: String,
    pub status: String,
}