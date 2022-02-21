use gm::utils::dates::from_datetimestr_to_option_naivedatetime;

pub const CLICK_COOKIE_NAME: &'static str = "gm-affiliate-click";

#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Affiliate {
    pub affiliate_id: String,
    pub user_id: String,
    #[serde(default)]
    #[serde(deserialize_with = "from_datetimestr_to_option_naivedatetime")]
    pub affiliate_created_at: Option<chrono::NaiveDateTime>,
    #[serde(default)]
    #[serde(deserialize_with = "from_datetimestr_to_option_naivedatetime")]
    pub affiliate_updated_at: Option<chrono::NaiveDateTime>,
    pub is_deleted: bool,
    pub is_seller_program_enabled: bool,
    pub is_buyer_program_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConversionType {
    ORDER_CONFIRM
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionRecord {
    pub conversion_id: String,
    pub affiliate_id: String,
    #[serde(default)]
    #[serde(deserialize_with = "from_datetimestr_to_option_naivedatetime")]
    pub clickCreatedAt: Option<chrono::NaiveDateTime>,
    pub conversion_type: ConversionType,
    pub entity_id: Option<String>,
    pub click_id: String,
}


#[test]
fn test_deserializes_affiliate() {

    let test_str = r#"
    {
        "affiliateId": "ascv57g",
        "userId": "user_c40b453a-2021-4f19-ac4f-023fc3e7eebd",
        "affiliateCreatedAt": "2020-05-09 02:06:05.494",
        "affiliateUpdatedAt": "2020-05-09 02:06:05.494",
        "isDeleted": false,
        "isSellerProgramEnabled": true,
        "isBuyerProgramEnabled": true
    }
    "#;

    let res = serde_json::from_str::<Affiliate>(test_str);
    match res {
        Ok(affiliate) => {
            assert_eq!(
                affiliate.affiliate_id,
                String::from("ascv57g")
            );
        },
        Err(e) => panic!(e.to_string()),
    };
}