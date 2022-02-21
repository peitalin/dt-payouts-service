use diesel::prelude::*;
use diesel::sql_types::{Double, Float8, Jsonb, Json, Text, BigInt, Timestamp, Nullable};
use diesel::serialize::{Output, ToSql};
use diesel::pg::Pg;
use diesel::deserialize::FromSql;
use std::str::FromStr;

use gm::db::schema::payout_splits;
use itertools::zip;
use std::collections::HashMap;

use chrono::prelude::Utc;
use chrono::Datelike;
use chrono::offset::TimeZone;

use crate::db;
use crate::pricing::{
    calculate_platform_fees,
    PaymentFees,
    CalculatedEarnings,
};
use crate::models::{
    OrderItemRpc,
    PayoutItem,
    PayoutDealType,
    PayoutSplit,
    PayeeType,
};



pub fn to_payout_items(
    conn: &diesel::PgConnection,
    order_items_rpc: Vec<OrderItemRpc>,
    tx_id: &str,
    // supplied by OrderItems, or set by Mock tests
    created_at: &chrono::NaiveDateTime,
    buyer_affiliate_user_id: Option<String>,
) -> Vec<PayoutItem> {

    debug!("\n\n============= to_payout_items(...) =================\n");
    // 1. lookup the most current PayoutSplit for buyer_affiliate
    let buyer_aff_psplit: Option<PayoutSplit> = match buyer_affiliate_user_id {
        None => None,
        Some(ba_user_id) => {
            db::try_read_buyer_psplit_create_on_null(conn, ba_user_id)
        }
    };
    debug!("Buyer Affiliate PayoutSplit: {:?}", &buyer_aff_psplit);
    // do this outside order_items_rpc.iter(), as buyer_affiliate applies to all
    // orderItems, instead of each orderItem having it's own buyer-affiliate

    // // 2. Lookup PayoutSplits for all sellers and their respective
    // // seller_affiliate across all orderItems (by orderItem.storeId)
    // let store_ids = order_items_rpc
    //     .iter()
    //     .map(|oitem: &OrderItemRpc| oitem.store_id.clone())
    //     .collect::<Vec<String>>();

    // // Each orderItem has a seller, and each seller may have a seller_affiliate
    // // who referred them.
    // let seller_aff_psplit_hmap: HashMap<String, PayoutSplitSellerAndAffiliate> =
    // db::read_current_seller_referrer_payout_splits_by_store_ids(
    //     &conn,
    //     store_ids.clone(),
    // );
    // debug!("HashMap<storeOrUserId, PayoutSplit>: {:?}", &seller_aff_psplit_hmap);


    order_items_rpc.clone()
    .iter()
    .map(|oitem: &OrderItemRpc| {

        // seller pays payment_processing_fees
        let seller_payment_proc_fee = PaymentFees::new()
            .calculate_payment_processing_fee(oitem.actual_price);

        // Each orderItem has a seller, and each seller may have a seller_affiliate
        // who referred them.
        // 2. lookup PayoutSplits for both seller and seller_affiliate
        // in this particular orderItem
        debug!("Looking up PayoutSplits for storeId: {:?}", &oitem.store_id);
        let (
            seller_psplit,
            seller_aff_psplit
        ): (Option<PayoutSplit>, Option<PayoutSplit>) =
        match db::read_current_seller_referrer_payout_splits_by_store_id(
            &conn,
            oitem.store_id.clone()
        ) {
            None => (None, None),
            Some(psplits) => {
                (psplits.referred_seller, psplits.seller_affiliate)
            }
        };

        debug!("Seller PayoutSplit: {:?}", &seller_psplit);
        debug!("Seller Affiliate PayoutSplit: {:?}", &seller_aff_psplit);

        debug!("\n====================================");
        debug!("Calculating Earnings from PayoutSplits");
        let CalculatedEarnings {
            seller_earnings_less_payment_fee,
            payment_processing_fee,
            gm_earnings,
            buyer_affiliate_earnings,
            seller_affiliate_earnings,
        } = calculate_platform_fees(
            oitem.actual_price,
            seller_payment_proc_fee,
            seller_psplit, // PayputSplit for Seller goes here
            buyer_aff_psplit.clone(), // PayoutSplit goes here
            seller_aff_psplit.clone(), // PayoutSplit goes here
            Some(created_at.clone())
        );
        debug!("seller_earnings_less_payment_fee: {:?}", &seller_earnings_less_payment_fee);
        debug!("gm_earnings: {:?}", &gm_earnings);
        debug!("buyer_affiliate_earnings: {:?}", &buyer_affiliate_earnings);
        debug!("seller_affiliate_earnings: {:?}", &seller_affiliate_earnings);

        let mut pitems = vec![
            // STORE
            PayoutItem::new(
                oitem.id.clone(),
                oitem.store_id.clone(),
                Some(PayeeType::STORE),
                seller_earnings_less_payment_fee,
                payment_processing_fee, // seller pays payment processing fee
                created_at.clone(),
                oitem.currency.clone(),
                tx_id.to_string(),
            ),
            // PLATFORM
            PayoutItem::new(
                oitem.id.clone(),
                String::from("gm-platform"),
                Some(PayeeType::PLATFORM),
                gm_earnings,
                0, // payment_processing_fee paid by platform
                created_at.clone(),
                oitem.currency.clone(),
                tx_id.to_string(),
            ),
        ];

        if let Some(b) = buyer_aff_psplit.clone() {
            pitems.append(&mut vec![
                // BUYER AFFILIATE, filtered out if 0
                PayoutItem::new(
                    oitem.id.clone(),
                    b.store_or_user_id.clone(),
                    Some(PayeeType::BUYER_AFFILIATE),
                    buyer_affiliate_earnings,
                    0, // payment_processing_fee paid by buyer affiliate
                    created_at.clone(),
                    oitem.currency.clone(),
                    tx_id.to_string(),
                )
            ])
        };

        if let Some(s) = seller_aff_psplit {
            pitems.append(&mut vec![
                // SELLER AFFILIATE, filtered out if 0
                PayoutItem::new(
                    oitem.id.clone(),
                    s.store_or_user_id.clone(),
                    Some(PayeeType::SELLER_AFFILIATE),
                    seller_affiliate_earnings,
                    0, // payment_processing_fee paid by seller affiliate
                    created_at.clone(),
                    oitem.currency.clone(),
                    tx_id.to_string(),
                )
            ])
        };

        // return newly generated payout_items
        pitems
    })
    .flatten()
    .filter(|pItem: &PayoutItem| pItem.amount > 0)
    .collect::<Vec<PayoutItem>>()
}


#[derive(Clone, Debug, Serialize, Deserialize, QueryableByName)]
pub struct PayoutSplitSellerAndAffiliate {
    #[sql_type = "Nullable<Json>"]
    pub referred_seller: Option<PayoutSplit>,
    #[sql_type = "Nullable<Json>"]
    pub seller_affiliate: Option<PayoutSplit>,
}
impl PayoutSplitSellerAndAffiliate {
    pub fn new() -> Self {
        Self {
            referred_seller: None,
            seller_affiliate: None,
        }
    }
}

