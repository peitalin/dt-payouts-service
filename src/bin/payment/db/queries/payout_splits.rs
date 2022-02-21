use diesel::prelude::*;
use diesel::result::Error;
use diesel::PgConnection;
use diesel::sql_types::{ Text, Timestamp, Array };
use gm::db;

use crate::models::{
    PayoutSplit,
    ErrJson,
    DbError,
};
use crate::models::{
    Payout,
    PayoutStatus,
    PayoutPeriod,
    PayoutSplitSellerAndAffiliate,
    PayoutDealType,
};
use itertools::zip;
use std::collections::HashMap;
use crate::pricing::BUYER_AFFILIATE_FEE_PERCENTAGE;


////////////////////////
/// Payout Methods
////////////////////////


pub fn write_payout_split(
    conn: &PgConnection,
    payout_split: PayoutSplit,
) -> Result<PayoutSplit, DbError> {

    use db::schema::payout_splits;

    diesel::insert_into(payout_splits::table)
        .values(payout_split)
        .get_result::<PayoutSplit>(conn)
        .map_err(|e| DbError::PayoutSplitWriteError(errJson!(e)))
}

pub fn write_two_payout_splits(
    conn: &PgConnection,
    referred_seller_payout_split: PayoutSplit,
    seller_affiliate_payout_split: PayoutSplit,
) -> Result<(PayoutSplit, PayoutSplit), DbError> {

    use db::schema::payout_splits;
    // both succeed or none do
    conn.transaction::<(PayoutSplit, PayoutSplit), diesel::result::Error, _>(|| {

        let referred_seller_ps = diesel::insert_into(payout_splits::table)
            .values(referred_seller_payout_split)
            .get_result::<PayoutSplit>(conn)?;

        let seller_affiliate_ps = diesel::insert_into(payout_splits::table)
            .values(seller_affiliate_payout_split)
            .get_result::<PayoutSplit>(conn)?;

        // return Result<(PayoutSplit, PayoutSplit), Error>
        Ok((referred_seller_ps, seller_affiliate_ps))

    }).map_err(|e| DbError::PayoutSplitWriteError(errJson!(e)))
}

pub fn read_payout_split(
    conn: &PgConnection,
    payout_split_id: String,
) -> Result<PayoutSplit, DbError> {

    use db::schema::payout_splits;

    payout_splits::table
        .filter(payout_splits::id.eq(payout_split_id))
        .get_result::<PayoutSplit>(conn)
        .map_err(|e| DbError::PayoutSplitReadError(errJson!(e)))
}


pub fn read_payout_splits_by_ids(
    conn: &PgConnection,
    payout_split_ids: Vec<String>,
) -> Result<Vec<PayoutSplit>, DbError> {

    use db::schema::payout_splits;

    payout_splits::table
        .filter(payout_splits::id.eq_any(payout_split_ids))
        .load::<PayoutSplit>(conn)
        .map_err(|e| DbError::PayoutSplitReadError(errJson!(e)))
}


pub fn read_payout_splits_of_user_id(
    conn: &PgConnection,
    user_id: String,
    payout_deal_types: Option<Vec<PayoutDealType>>,
) -> Result<Vec<PayoutSplit>, DbError> {

    use db::schema::payout_splits;

    match payout_deal_types {
        None => {
            payout_splits::table
                .filter(
                    payout_splits::store_or_user_id.eq(user_id)
                )
                .load::<PayoutSplit>(conn)
                .map_err(|e| DbError::PayoutSplitReadError(errJson!(e)))
        },
        Some(deal_types) => {
            payout_splits::table
                .filter(
                    payout_splits::store_or_user_id.eq(user_id)
                    .and(payout_splits::deal_type.eq_any(deal_types))
                )
                .load::<PayoutSplit>(conn)
                .map_err(|e| DbError::PayoutSplitReadError(errJson!(e)))
        }
    }
}


pub fn read_current_payout_splits_by_store_or_user_ids(
    conn: &PgConnection,
    store_or_user_ids: &Vec<String>,
    payout_deal_types: Option<Vec<PayoutDealType>>,
) -> Result<Vec<PayoutSplit>, DbError> {

    use db::schema::payout_splits;

    let payout_deal_types_arg = match payout_deal_types {
        Some(p) => p,
        None => vec! [
            PayoutDealType::BUYER_AFFILIATE,
            PayoutDealType::SELLER,
            PayoutDealType::SELLER_AFFILIATE,
            PayoutDealType::REFERRED_SELLER,
        ]
    };

    let result: Result<Vec<PayoutSplit>, DbError> = diesel::sql_query(format!(r#"
        SELECT rank_payout_splits_by_created_at_filter.* FROM (
            SELECT payout_splits.*,
            rank() OVER (
                PARTITION BY store_or_user_id
                ORDER BY created_at DESC
            )
            FROM payout_splits
            WHERE deal_type = ANY($2)
        ) rank_payout_splits_by_created_at_filter
        WHERE RANK = 1
        AND store_or_user_id = ANY($1)
    "#))
    .bind::<Array<Text>, _>(store_or_user_ids)
    .bind::<Array<Text>, _>(payout_deal_types_arg)
    .load(conn)
    .map_err(|e| DbError::PayoutSplitReadError(errJson!(e)));
    // cannot do .load::<Vec<PayoutSplit>>(), it will
    // run into type/trait error. Trait not defined for Array<Text>
    // So set result type: Result<Vec<PayoutSplit>, DbError> and return result
    result
}


pub fn try_read_buyer_psplit_create_on_null(
    conn: &PgConnection,
    buyer_affiliate_user_id: String,
) -> Option<PayoutSplit> {

    let maybe_read_psplit = read_current_payout_splits_by_store_or_user_ids(
        conn,
        &vec![buyer_affiliate_user_id.clone()],
        Some(vec![PayoutDealType::BUYER_AFFILIATE]),
    ).ok();

    // if BUYER_AFFILIATE PayoutSplit exists, return early with it
    if let Some(vpsplit) = maybe_read_psplit {
        if let Some(psplit) = vpsplit.into_iter().next() {
            return Some(psplit)
        }
    };

    // otherwise write a BUYER_AFFILIATE PayoutSplit for this userId
    // and return it
    let new_ba_payout_split = PayoutSplit::new(
        buyer_affiliate_user_id,
        PayoutDealType::BUYER_AFFILIATE,
        None, // Buyer affiliate deal never expires
        BUYER_AFFILIATE_FEE_PERCENTAGE, // Default rate for Buyer Affiliates
        None,
    );

    write_payout_split(conn, new_ba_payout_split).ok()
}


pub fn read_current_seller_referrer_payout_splits_by_store_id(
    conn: &PgConnection,
    store_or_user_id: String,
) -> Option<PayoutSplitSellerAndAffiliate> {

    use db::schema::payout_splits;
    debug!(
        "Getting REFERRED_SELLER and SELLER_AFFILIATE payout splits for: {:?}",
        &store_or_user_id
    );

    let result: Result<PayoutSplitSellerAndAffiliate, DbError> =
    diesel::sql_query(format!(r#"

        WITH current_psplits AS (
            SELECT rank_payout_splits_by_created_at_filter.* FROM (
                SELECT
                    payout_splits.*,
                    rank() OVER (
                        PARTITION BY store_or_user_id
                        ORDER BY created_at DESC
                    )
                FROM payout_splits
                WHERE payout_splits.deal_type = 'REFERRED_SELLER'
                    OR payout_splits.deal_type = 'SELLER'
            ) rank_payout_splits_by_created_at_filter
            WHERE RANK = 1
            AND store_or_user_id = $1
        ),

        current_aff_psplits AS (
            SELECT *
            FROM payout_splits
            WHERE payout_splits.deal_type = 'SELLER_AFFILIATE'
        )

        SELECT
            json_build_object(
                'id', current_psplits.id,
                'createdAt', current_psplits.created_at,
                'storeOrUserId', current_psplits.store_or_user_id,
                'dealType', current_psplits.deal_type,
                'expiresAt', current_psplits.expires_at,
                'rate', current_psplits.rate,
                'referrerId', current_psplits.referrer_id
                /* 'rank', current_psplits.rank */
            ) as referred_seller,

			CASE
				WHEN payout_splits.id IS NOT NULL
				THEN
					json_build_object(
		                'id', payout_splits.id,
		                'createdAt', payout_splits.created_at,
		                'storeOrUserId', payout_splits.store_or_user_id,
		                'dealType', payout_splits.deal_type,
		                'expiresAt', payout_splits.expires_at,
		                'rate', payout_splits.rate,
		                'referrerId', payout_splits.referrer_id
		                /* 'rank', payout_splits.rank */
		            )
				ELSE null
			END as seller_affiliate

        FROM current_psplits
        LEFT JOIN current_aff_psplits payout_splits
            ON payout_splits.id = current_psplits.referrer_id

    "#))
    .bind::<Text, _>(&store_or_user_id)
    .get_result(conn)
    .map_err(|e| DbError::PayoutSplitReadError(errJson!(e)));
    // cannot do .load::<Vec<PayoutSplit>>(), it will
    // run into type/trait error. Trait not defined for Array<Text>
    // So set result type: Result<Vec<PayoutSplit>, DbError> and return result

    match result {
        Err(_e) => {
            debug!("Could not read PayoutSplits for ID: {:?}", &store_or_user_id);
            None
        },
        Ok(payout_split) => {
            Some(payout_split)
        }
    }
}


// pub fn read_current_seller_referrer_payout_splits_by_store_ids(
//     conn: &PgConnection,
//     store_or_user_ids: Vec<String>,
// ) -> HashMap<String, PayoutSplitSellerAndAffiliate> {

//     use db::schema::payout_splits;
//     debug!(
//         "Getting REFERRED_SELLER and SELLER_AFFILIATE payout splits for: {:?}",
//         &store_or_user_ids
//     );

//     let result: Result<Vec<PayoutSplitSellerAndAffiliate>, DbError> =
//     diesel::sql_query(format!(r#"

//         WITH current_psplits AS (
//             SELECT rank_payout_splits_by_created_at_filter.* FROM (
//                 SELECT
//                     payout_splits.*,
//                     rank() OVER (
//                         PARTITION BY store_or_user_id
//                         ORDER BY created_at DESC
//                     )
//                 FROM payout_splits
//                 WHERE payout_splits.deal_type = 'REFERRED_SELLER'
//                     OR payout_splits.deal_type = 'SELLER'
//             ) rank_payout_splits_by_created_at_filter
//             WHERE RANK = 1
//             AND store_or_user_id = ANY($1)
//         ),

//         current_aff_psplits AS (
//             SELECT *
//             FROM payout_splits
//             WHERE payout_splits.deal_type = 'SELLER_AFFILIATE'
//         )

//         SELECT
//             json_build_object(
//                 'id', current_psplits.id,
//                 'createdAt', current_psplits.created_at,
//                 'storeOrUserId', current_psplits.store_or_user_id,
//                 'dealType', current_psplits.deal_type,
//                 'expiresAt', current_psplits.expires_at,
//                 'rate', current_psplits.rate,
//                 'referrerId', current_psplits.referrer_id
//                 /* 'rank', current_psplits.rank */
//             ) as referred_seller,

// 			CASE
// 				WHEN payout_splits.id IS NOT NULL
// 				THEN
// 					json_build_object(
// 		                'id', payout_splits.id,
// 		                'createdAt', payout_splits.created_at,
// 		                'storeOrUserId', payout_splits.store_or_user_id,
// 		                'dealType', payout_splits.deal_type,
// 		                'expiresAt', payout_splits.expires_at,
// 		                'rate', payout_splits.rate,
// 		                'referrerId', payout_splits.referrer_id
// 		                /* 'rank', payout_splits.rank */
// 		            )
// 				ELSE null
// 			END as seller_affiliate

//         FROM current_psplits
//         LEFT JOIN current_aff_psplits payout_splits
//             ON payout_splits.id = current_psplits.referrer_id

//     "#))
//     .bind::<Array<Text>, _>(&store_or_user_ids)
//     .load(conn)
//     .map_err(|e| DbError::PayoutSplitReadError(errJson!(e)));
//     // cannot do .load::<Vec<PayoutSplit>>(), it will
//     // run into type/trait error. Trait not defined for Array<Text>
//     // So set result type: Result<Vec<PayoutSplit>, DbError> and return result

//     match result {
//         Err(_e) => {
//             debug!("Could not read PayoutSplits for IDs: {:?}", &store_or_user_ids);
//             debug!("Returning an empty HashMap");
//             HashMap::new()
//         },
//         Ok(payout_splits) => {
//             // convert to HashMap<StoreId, PayoutSplit>
//             zip(
//                 store_or_user_ids.into_iter(), // StoreId params
//                 payout_splits.into_iter() // PayoutSplits Read from DB
//             ).collect::<HashMap<String, PayoutSplitSellerAndAffiliate>>()
//             // StoreOrUserIds may be larger than PayoutSplits if a particular StoreId
//             // does not have a PayoutSPlit in the DB (returns null)
//             // Then storeOrUserIds.len() > payout_splits.len() and the resulting HashMap
//             // has storeIds that point to the wrong PayoutSplit
//         }
//     }
// }

/// Only used for testing. In production you never delete payoutSplits
/// because it would just revert to the previous dated payoutSplits
pub fn delete_payout_split(
    conn: &PgConnection,
    payout_split_id: &str,
) -> Result<PayoutSplit, DbError> {

    use db::schema::payout_splits;

    diesel::delete(payout_splits::table
        .filter(payout_splits::id.eq(payout_split_id)))
        .get_result::<PayoutSplit>(conn)
        .map_err(|e| DbError::PayoutSplitWriteError(errJson!(e)))

}

pub fn delete_all_payout_splits_for_user_id(
    conn: &PgConnection,
    user_id: &str,
) -> Result<PayoutSplit, DbError> {

    use db::schema::payout_splits;

    diesel::delete(payout_splits::table
        .filter(payout_splits::store_or_user_id.eq(user_id)))
        .get_result::<PayoutSplit>(conn)
        .map_err(|e| DbError::PayoutSplitWriteError(errJson!(e)))
}
