use gm::utils::dates::from_timestamp_s_to_naivedatetime;
use gm::utils::dates::from_datetimestr_to_naivedatetime;
use gm::utils::dates::from_datetimestr_to_option_naivedatetime;
use chrono::Utc;
use crate::models::PayoutItem;


#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderDb {
    pub id: String,
    pub order_snapshot_id: Option<String>,
    pub order_item_ids: Option<Vec<String>>,
    pub payout_items: Option<Vec<OrderItemRpc>>,
    #[serde(deserialize_with = "from_datetimestr_to_option_naivedatetime")]
    pub created_at: Option<chrono::NaiveDateTime>,
    #[serde(deserialize_with = "from_datetimestr_to_option_naivedatetime")]
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attached_promo_code_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_snapshot: Option<OrderSnapshotDb>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_snapshot: Option<OrderSnapshotDb>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub historical_snapshots: Option<Vec<OrderSnapshotDb>>,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderSnapshotDb {
    pub id: String,
    pub order_id: String,
    pub order_status: String,
    #[serde(deserialize_with = "from_datetimestr_to_option_naivedatetime")]
    pub created_at: Option<chrono::NaiveDateTime>,
    pub currency: String,
    pub subtotal: i32,
    pub taxes: i32,
    pub payment_processing_fee: i32,
    pub total: i32,
    pub transaction_id: String,
    pub payment_processor: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderId {
    pub order_id: String,
}


#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderItemRpc {
    pub id: String,
    pub actual_price: i32,
    #[serde(deserialize_with = "from_datetimestr_to_naivedatetime")]
    pub created_at: chrono::NaiveDateTime,
    pub currency: String,
    /// Only if buyer pays payment processing fee, otherwise seller pays
    /// and seller payment determined here in payment service
    pub payment_processing_fee: Option<i32>,
    /// entities
    pub store_id: String,
}



/////// Tests //////////
mod tests {
    use super::*;

    #[test]
    fn deserializes_orderDb() {
        let test_str = r#"
        {
            "id": "order_44605b12-1c5e-436c-90a4-9ac60df438af",
            "createdAt": "2019-10-18T14:51:28.398Z",
            "updatedAt": "2019-10-18T14:51:28.398Z",
            "userId": "0e4a5c3b-0d33-4e8f-877b-43170ac8c9b5",
            "orderSnapshotId": "order_snapshot_410fce41-9461-4b87-81e1-5e7651abaf52",
            "orderItemIds": [ "oitem_74621900-1338-4725-bbf3-dae383bc2ddf" ],
            "payoutItems": [
                {
                    "id": "oitem_74621900-1338-4725-bbf3-dae383bc2ddf",
                    "actualPrice": 1200,
                    "storeId": "store_123",
                    "seller_affiliate_id": "aff_seller_123123",
                    "buyer_affiliate_id": "aff_buyer_123123",
                    "createdAt": "2019-10-07T14:30:08.052Z",
                    "currency": "USD"
                }
            ],
            "appliedDiscountCodeIds": [],
            "currentSnapshot": {
                "id": "order_snapshot_410fce41-9461-4b87-81e1-5e7651abaf52",
                "orderId": "order_44605b12-1c5e-436c-90a4-9ac60df438af",
                "orderStatus": "CREATED",
                "createdAt": "2019-10-07T14:30:08.052Z",
                "currency": "USD",
                "subtotal": 1800,
                "taxes": 190,
                "paymentProcessingFee": 90,
                "total": 1800,
                "transactionId": "pi_1FQxDuKqy1M9WH1DMWpuXvfy",
                "paymentProcessor": "Stripe"
            }
        }
        "#;
        let res = serde_json::from_str::<OrderDb>(test_str);
        match res {
            Err(e) => panic!(e.to_string()),
            Ok(order) => {
                assert_eq!(
                    order.id,
                    String::from("order_44605b12-1c5e-436c-90a4-9ac60df438af")
                );
                // println!("CREATEDAT: {:?}", order.created_at);
                assert_eq!(
                    order.payout_items.unwrap().iter().next().unwrap().store_id,
                    String::from("store_123")
                );
            }
        }
    }

    #[test]
    fn deserializes_orderDb_missing_applied_discount_codes() {
        let test_str = r#"
        {
            "id": "order_44605b12-1c5e-436c-90a4-9ac60df438af",
            "createdAt": "2019-10-07T14:30:08.052Z",
            "updatedAt": "2019-10-07T14:30:08.052Z",
            "userId": "0e4a5c3b-0d33-4e8f-877b-43170ac8c9b5",
            "orderSnapshotId": "order_snapshot_410fce41-9461-4b87-81e1-5e7651abaf52",
            "orderItemIds": [ "oitem_74621900-1338-4725-bbf3-dae383bc2ddf" ],
            "payoutItems": [
                {
                    "id": "oitem_74621900-1338-4725-bbf3-dae383bc2ddf",
                    "actualPrice": 1200,
                    "storeId": "store_123",
                    "createdAt": "2019-10-07T14:30:08.052Z",
                    "currency": "USD"
                }
            ],
            "currentSnapshot": {
                "id": "order_snapshot_410fce41-9461-4b87-81e1-5e7651abaf52",
                "orderId": "order_44605b12-1c5e-436c-90a4-9ac60df438af",
                "orderStatus": "CREATED",
                "createdAt": "2019-10-07T14:30:08.052Z",
                "currency": "USD",
                "subtotal": 1800,
                "taxes": 190,
                "paymentProcessingFee": 90,
                "total": 1800,
                "transactionId": "pi_1FQxDuKqy1M9WH1DMWpuXvfy",
                "paymentProcessor": "Stripe"
            }
        }
        "#;

        let res = serde_json::from_str::<OrderDb>(test_str);
        match res {
            Err(e) => panic!(e.to_string()),
            Ok(order) => {
                assert_eq!(
                    order.id,
                    String::from("order_44605b12-1c5e-436c-90a4-9ac60df438af")
                );
                assert_eq!(
                    order.payout_items.unwrap().iter().next().unwrap().store_id,
                    String::from("store_123")
                );
            }
        }
    }

}