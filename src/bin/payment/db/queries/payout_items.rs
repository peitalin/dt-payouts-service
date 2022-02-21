use diesel::prelude::*;
use diesel::result::Error;
use diesel::PgConnection;
use diesel::sql_types::{ Text, Timestamp, Array };
use gm::db;
use chrono::Datelike;

use crate::models::{
    StripeError,
    DbError,
    ErrJson,
    Transaction,
    CartDb,
    PaymentMethodDb,
    PaymentMethodAddress,
    Refund,
    PayoutItem,
    PayoutItemAggregates,
    Payout,
    PayoutStatus,
    PayoutItemHistorySummaries,
    SummaryStatistics,
    PayeeType,
};
use crate::models::{
    ConnectionQuery,
    PageBasedConnectionQuery,
};
use crate::models::paginate_page::{
    PaginatedPage,
    PaginatePage,
};
use crate::models::paginate_cursor::*;


////////////////////////
/// PayoutItems
////////////////////////


pub fn write_payout_items(
    conn: &PgConnection,
    payout_items: &Vec<PayoutItem>,
) -> Result<Vec<PayoutItem>, DbError> {

    use db::schema::payout_items;

    diesel::insert_into(payout_items::table)
        .values(payout_items)
        .load::<PayoutItem>(conn)
        .map_err(|e| DbError::PayoutItemWriteError(errJson!(e)))
}


pub fn delete_payout_items(
    conn: &PgConnection,
    payout_item_ids: &Vec<String>,
) -> Result<Vec<PayoutItem>, DbError> {

    use db::schema::payout_items;

    diesel::delete(payout_items::table
        .filter(payout_items::id.eq_any(payout_item_ids)))
        .load::<PayoutItem>(conn)
        .map_err(|e| DbError::PayoutItemWriteError(errJson!(e)))
}



pub fn update_payout_items_status(
    conn: &PgConnection,
    payout_items_ids: &Vec<String>,
    payout_status: PayoutStatus,
    payout_id: &str,
) -> Result<Vec<PayoutItem>, DbError> {

    use db::schema::payout_items;

    diesel::update(
        payout_items::table
            .filter(payout_items::id.eq_any(payout_items_ids))
    )
    .set((
        payout_items::payout_status.eq(payout_status.as_string()),
        payout_items::payout_id.eq(payout_id),

    ))
    .load::<PayoutItem>(conn)
    .map_err(|e| DbError::PayoutItemWriteError(errJson!(e)))
}


pub fn read_payout_items_by_ids(
    conn: &PgConnection,
    payout_item_ids: &Vec<String>,
) -> Result<Vec<PayoutItem>, DbError> {

    use db::schema::payout_items;
    use diesel::dsl::*;

    payout_items::table
        .filter(payout_items::id.eq_any(payout_item_ids))
        .load::<PayoutItem>(conn)
        .map_err(|e| DbError::PayoutItemReadError(errJson!(e)))
}


pub fn read_payout_items_by_order_item_ids(
    conn: &PgConnection,
    order_item_ids: &Vec<String>,
) -> Result<Vec<PayoutItem>, DbError> {
    use db::schema::payout_items;
    use diesel::dsl::*;

    payout_items::table
        .filter(payout_items::order_item_id.eq_any(order_item_ids))
        .load::<PayoutItem>(conn)
        .map_err(|e| DbError::PayoutItemReadError(errJson!(e)))
}


pub fn read_payout_items_in_period(
    conn: &PgConnection,
    start_date: chrono::NaiveDateTime,
    end_date: chrono::NaiveDateTime,
    statuses: Option<Vec<PayoutStatus>>,
    store_id: Option<String>,
) -> Result<Vec<PayoutItem>, DbError> {

    use db::schema::payout_items;
    use diesel::dsl::*;

    match (statuses, store_id) {
        (Some(pstatuses), Some(storeId)) => {
            payout_items::table
                .filter(
                    payout_items::created_at.gt(start_date)
                    .and(payout_items::created_at.lt(end_date))
                    .and(payout_items::payout_status.eq_any(&pstatuses))
                    .and(payout_items::payee_id.eq(storeId))
                )
                .load::<PayoutItem>(conn)
                .map_err(|e| DbError::PayoutItemReadError(errJson!(e)))
        },
        (None, Some(storeId)) => {
            payout_items::table
                .filter(
                    payout_items::created_at.gt(start_date)
                    .and(payout_items::created_at.lt(end_date))
                    .and(payout_items::payee_id.eq(storeId))
                )
                .load::<PayoutItem>(conn)
                .map_err(|e| DbError::PayoutItemReadError(errJson!(e)))
        },
        (Some(pstatuses), None) => {
            payout_items::table
                .filter(
                    payout_items::created_at.gt(start_date)
                    .and(payout_items::created_at.lt(end_date))
                    .and(payout_items::payout_status.eq_any(&pstatuses))
                )
                .load::<PayoutItem>(conn)
                .map_err(|e| DbError::PayoutItemReadError(errJson!(e)))
        },
        (None, None) => {
            payout_items::table
                .filter(
                    payout_items::created_at.gt(start_date)
                    .and(payout_items::created_at.lt(end_date))
                )
                .load::<PayoutItem>(conn)
                .map_err(|e| DbError::PayoutItemReadError(errJson!(e)))
        },
    }
}


pub fn read_payout_items_in_period_paginate_by_cursor(
    conn: &PgConnection,
    start_date: chrono::NaiveDateTime,
    end_date: chrono::NaiveDateTime,
    status: Option<PayoutStatus>,
    connection_query: ConnectionQuery,
) -> Result<(Vec<PayoutItem>, i64, bool), DbError> {

    use db::schema::payout_items;
    use diesel::dsl::*;
    // paginate must come after .select()
    // select() implements Query, etc
    let orderField = String::from("created_at");

    let maybe_cursor = match connection_query.cursor.clone() {
        None => None,
        Some(c) => decode_datetime_cursor(&c).ok(),
    };

    let PageDirection { lessThan, queryAscending } = get_page_direction(
        connection_query.sortAscending.unwrap_or(false),
        connection_query.pageBackwards.unwrap_or(false)
    );

    match (status, maybe_cursor, lessThan) {
        (Some(payout_status), Some(cursor), true) => {
            payout_items::table
                .select(payout_items::all_columns)
                .filter(
                    payout_items::created_at.gt(start_date)
                    .and(payout_items::created_at.lt(end_date))
                    .and(payout_items::payout_status.eq(payout_status))
                    // cursor: lessThan
                    .and(payout_items::created_at.lt(cursor.value))
                )
                .paginate_by_cursor(
                    orderField,
                    Some(queryAscending),
                    connection_query.pageBackwards,
                )
                .per_page(connection_query.count)
                .load_and_count_pages::<PayoutItem>(conn)
                .map_err(|e| DbError::PayoutItemReadError(errJson!(e)))
        },
        (Some(payout_status), Some(cursor), false) => {
            payout_items::table
                .select(payout_items::all_columns)
                .filter(
                    payout_items::created_at.gt(start_date)
                    .and(payout_items::created_at.lt(end_date))
                    .and(payout_items::payout_status.eq(payout_status))
                    // cursor: greaterThan
                    .and(payout_items::created_at.gt(cursor.value))
                )
                .paginate_by_cursor(
                    orderField,
                    Some(queryAscending),
                    connection_query.pageBackwards,
                )
                .per_page(connection_query.count)
                .load_and_count_pages::<PayoutItem>(conn)
                .map_err(|e| DbError::PayoutItemReadError(errJson!(e)))
        },
        (None, Some(cursor), true) => {
            payout_items::table
                .select(payout_items::all_columns)
                .filter(
                    payout_items::created_at.gt(start_date)
                    .and(payout_items::created_at.lt(end_date))
                    .and(payout_items::created_at.lt(cursor.value))
                )
                .paginate_by_cursor(
                    orderField,
                    Some(queryAscending),
                    connection_query.pageBackwards,
                )
                .per_page(connection_query.count)
                .load_and_count_pages::<PayoutItem>(conn)
                .map_err(|e| DbError::PayoutItemReadError(errJson!(e)))
        },
        (None, Some(cursor), false) => {
            payout_items::table
                .select(payout_items::all_columns)
                .filter(
                    payout_items::created_at.gt(start_date)
                    .and(payout_items::created_at.lt(end_date))
                    .and(payout_items::created_at.gt(cursor.value))
                )
                .paginate_by_cursor(
                    orderField,
                    Some(queryAscending),
                    connection_query.pageBackwards,
                )
                .per_page(connection_query.count)
                .load_and_count_pages::<PayoutItem>(conn)
                .map_err(|e| DbError::PayoutItemReadError(errJson!(e)))
        },
        (Some(payout_status), None, _) => {
            payout_items::table
                .select(payout_items::all_columns)
                .filter(
                    payout_items::created_at.gt(start_date)
                    .and(payout_items::created_at.lt(end_date))
                    .and(payout_items::payout_status.eq(payout_status))
                )
                .paginate_by_cursor(
                    orderField,
                    Some(queryAscending),
                    connection_query.pageBackwards,
                )
                .per_page(connection_query.count)
                .load_and_count_pages::<PayoutItem>(conn)
                .map_err(|e| DbError::PayoutItemReadError(errJson!(e)))
        },
        (None, None, _) => {
            payout_items::table
                .select(payout_items::all_columns)
                .filter(
                    payout_items::created_at.gt(start_date)
                    .and(payout_items::created_at.lt(end_date))
                )
                .paginate_by_cursor(
                    orderField,
                    Some(queryAscending),
                    connection_query.pageBackwards,
                )
                .per_page(connection_query.count)
                .load_and_count_pages::<PayoutItem>(conn)
                .map_err(|e| DbError::PayoutItemReadError(errJson!(e)))
        },
    }
}


pub fn read_payout_items_in_period_paginate_by_page(
    conn: &PgConnection,
    start_date: chrono::NaiveDateTime,
    end_date: chrono::NaiveDateTime,
    status: Option<PayoutStatus>,
    connection_query: PageBasedConnectionQuery,
) -> Result<(Vec<PayoutItem>, i64), DbError> {

    use db::schema::payout_items;
    use diesel::dsl::*;
    // paginate must come after .select()
    // select() implements Query, etc

    let PageDirection { lessThan: _, queryAscending } = get_page_direction(
        connection_query.sortAscending.unwrap_or(false),
        false,
    );

    match (status, queryAscending) {
        (Some(payout_status), false) => {
            payout_items::table
                .select(payout_items::all_columns)
                .filter(
                    payout_items::created_at.gt(start_date)
                    .and(payout_items::created_at.lt(end_date))
                    .and(payout_items::payout_status.eq(payout_status))
                )
                .order(payout_items::created_at.desc())
                .paginate_by_page(
                    connection_query.pageNumber,
                )
                .per_page(connection_query.count)
                .load_and_count_pages::<PayoutItem>(conn)
                .map_err(|e| DbError::PayoutItemReadError(errJson!(e)))
        },
        (Some(payout_status), true) => {
            payout_items::table
                .select(payout_items::all_columns)
                .filter(
                    payout_items::created_at.gt(start_date)
                    .and(payout_items::created_at.lt(end_date))
                    .and(payout_items::payout_status.eq(payout_status))
                )
                .order(payout_items::created_at.asc())
                .paginate_by_page(
                    connection_query.pageNumber,
                )
                .per_page(connection_query.count)
                .load_and_count_pages::<PayoutItem>(conn)
                .map_err(|e| DbError::PayoutItemReadError(errJson!(e)))
        },
        (None, true) => {
            payout_items::table
                .select(payout_items::all_columns)
                .filter(
                    payout_items::created_at.gt(start_date)
                    .and(payout_items::created_at.lt(end_date))
                )
                .order(payout_items::created_at.asc())
                .paginate_by_page(
                    connection_query.pageNumber,
                )
                .per_page(connection_query.count)
                .load_and_count_pages::<PayoutItem>(conn)
                .map_err(|e| DbError::PayoutItemReadError(errJson!(e)))
        },
        (None, false) => {
            payout_items::table
                .select(payout_items::all_columns)
                .filter(
                    payout_items::created_at.gt(start_date)
                    .and(payout_items::created_at.lt(end_date))
                )
                .order(payout_items::created_at.desc())
                .paginate_by_page(
                    connection_query.pageNumber,
                )
                .per_page(connection_query.count)
                .load_and_count_pages::<PayoutItem>(conn)
                .map_err(|e| DbError::PayoutItemReadError(errJson!(e)))
        },
    }
}

pub fn read_payout_item_aggregates(
    conn: &PgConnection,
    start_date: chrono::NaiveDateTime,
    end_date: chrono::NaiveDateTime,
    sort_ascending: bool,
) -> PayoutItemAggregates {

    use db::schema::transactions;
    use diesel::dsl::*;

    let ascDesc = if sort_ascending { "ASC" } else { "DESC" };

    // Sum over payout_items.amount
    // Remember to deduce the paymentProcessingFees because Sellers pay for it

    let agg = diesel::sql_query(format!(r#"
        SELECT
            created_at,
            SUM(amount) OVER () + SUM(payment_processing_fee) OVER () as amount_total,
            SUM(payment_processing_fee) OVER () as fees_total,
            (SUM(CASE WHEN payout_status = 'UNPAID' OR payout_status = 'MISSING_PAYOUT_METHOD' THEN amount ELSE 0 END) OVER ()) as unpaid,
            COUNT(*) OVER ()
        FROM (
            SELECT * FROM payout_items
        ) q
        WHERE created_at > $1 AND created_at < $2
        ORDER BY created_at {}
        LIMIT 1
    "#, ascDesc))
    .bind::<Timestamp, _>(start_date)
    .bind::<Timestamp, _>(end_date)
    .get_result::<PayoutItemAggregates>(conn)
    .map_err(|e| DbError::PayoutItemReadError(errJson!(e)));

    debug!("read_payout_item_aggregates db result: {:?}", agg);

    match agg {
        Ok(res) => res,
        Err(_e) => PayoutItemAggregates::new(),
    }
}


pub fn read_payout_item_history_summaries(
    conn: &PgConnection,
    store_id: &str,
    payee_types: Option<Vec<PayeeType>>,
) -> Result<PayoutItemHistorySummaries, DbError> {

    use db::schema::payout_items;
    use diesel::dsl::*;
    use crate::models::PayoutPeriod;

    // Sum over payout_items.amount

    let current_year = chrono::Utc::now().year();
    let current_month: i32 = chrono::Utc::now().month() as i32;
    let current_payout_period = PayoutPeriod::new(current_year, current_month)
        .expect("impossible month for PayoutPeriod in payout_item_history_summaries");

    let last_month = match current_month {
        1 => 12, // current Month is Jan, so last month was December
        _ => current_month - 1,
    };
    let last_year = match last_month {
        12 => current_year - 1, // last month was December, so last year.
        _ => current_year,
    };
    let last_payout_period = PayoutPeriod::new(last_year, last_month)
        .expect("impossible month for PayoutPeriod in payout_item_history_summaries");

    let payee_types_params = payee_types.unwrap_or(vec![
        PayeeType::BUYER_AFFILIATE,
        PayeeType::SELLER_AFFILIATE,
        PayeeType::PLATFORM,
        PayeeType::STORE
    ]);
    // If not provided, default to returning payout items for all payee types

    // NOTE:
    // count is calculated as: SUM(CASE WHEN amount > 0 THEN 1 ELSE 0 END) OVER ()
    // because there are refund items which need to be skipped.
    // May want to write logic to subtract the order associated with refund.

    let agg = diesel::sql_query(format!(r#"
    SELECT
        (SELECT ROW_TO_JSON(x) FROM (
            SELECT
                (SUM(amount) OVER ()) AS amount_total,
                (SUM(CASE WHEN payout_status = 'UNPAID' OR payout_status = 'MISSING_PAYOUT_METHOD' THEN amount ELSE 0 END) OVER ()) as unpaid,
                (SUM(CASE WHEN amount > 0 THEN 1 ELSE 0 END) OVER ()) as count
            FROM payout_items
            WHERE created_at > (current_timestamp - interval '1 day')
                AND payout_items.payee_id = $1
                AND payout_items.payee_type = ANY($6)
            ORDER BY created_at DESC

            LIMIT 1
        ) x) as today,

        (SELECT ROW_TO_JSON(x) FROM (
            SELECT
                (SUM(amount) OVER ()) AS amount_total,
                (SUM(CASE WHEN payout_status = 'UNPAID' OR payout_status = 'MISSING_PAYOUT_METHOD' THEN amount ELSE 0 END) OVER ()) as unpaid,
                (SUM(CASE WHEN amount > 0 THEN 1 ELSE 0 END) OVER ()) as count
            FROM payout_items
            WHERE created_at > current_timestamp - interval '7 day'
                AND payout_items.payee_id = $1
                AND payout_items.payee_type = ANY($6)
            ORDER BY created_at DESC
            LIMIT 1
        ) x) as last_7_days,

        (SELECT ROW_TO_JSON(x) FROM (
            SELECT
                (SUM(amount) OVER ()) AS amount_total,
                (SUM(CASE WHEN payout_status = 'UNPAID' OR payout_status = 'MISSING_PAYOUT_METHOD' THEN amount ELSE 0 END) OVER ()) as unpaid,
                (SUM(CASE WHEN amount > 0 THEN 1 ELSE 0 END) OVER ()) as count
            FROM payout_items
            WHERE created_at > current_timestamp - interval '30 day'
                AND payout_items.payee_id = $1
                AND payout_items.payee_type = ANY($6)
            ORDER BY created_at DESC
            LIMIT 1
        ) x) as last_30_days,

        (SELECT ROW_TO_JSON(x) FROM (
            SELECT
                (SUM(amount) OVER ()) AS amount_total,
                (SUM(CASE WHEN payout_status = 'UNPAID' OR payout_status = 'MISSING_PAYOUT_METHOD' THEN amount ELSE 0 END) OVER ()) as unpaid,
                (SUM(CASE WHEN amount > 0 THEN 1 ELSE 0 END) OVER ()) as count
            FROM payout_items
            WHERE created_at > $2 AND created_at < $3
                AND payout_items.payee_id = $1
                AND payout_items.payee_type = ANY($6)
            ORDER BY created_at DESC
            LIMIT 1
        ) x) as last_period,

        (SELECT ROW_TO_JSON(x) FROM (
            SELECT
                (SUM(amount) OVER ()) AS amount_total,
                (SUM(CASE WHEN payout_status = 'UNPAID' OR payout_status = 'MISSING_PAYOUT_METHOD' THEN amount ELSE 0 END) OVER ()) as unpaid,
                (SUM(CASE WHEN amount > 0 THEN 1 ELSE 0 END) OVER ()) as count
            FROM payout_items
            WHERE created_at > $4 AND created_at < $5
                AND payout_items.payee_id = $1
                AND payout_items.payee_type = ANY($6)
            ORDER BY created_at DESC
            LIMIT 1
        ) x) as current_period,

        (SELECT ROW_TO_JSON(x) FROM (
            SELECT
                (SUM(amount) OVER ()) AS amount_total,
                (SUM(CASE WHEN payout_status = 'UNPAID' OR payout_status = 'MISSING_PAYOUT_METHOD' THEN amount ELSE 0 END) OVER ()) as unpaid,
                (SUM(CASE WHEN amount > 0 THEN 1 ELSE 0 END) OVER ()) as count
            FROM payout_items
            WHERE payout_items.payee_id = $1
                AND payout_items.payee_type = ANY($6)
            ORDER BY created_at DESC
            LIMIT 1
        ) x) as all_time

    FROM payout_items
    LIMIT 1
    "#))
    .bind::<Text, _>(store_id)
    .bind::<Timestamp, _>(last_payout_period.start_period)
    .bind::<Timestamp, _>(last_payout_period.end_period)
    .bind::<Timestamp, _>(current_payout_period.start_period)
    .bind::<Timestamp, _>(current_payout_period.end_period)
    .bind::<Array<Text>, _>(payee_types_params)
    .get_result::<PayoutItemHistorySummaries>(conn)
    .map_err(|e| DbError::PayoutItemReadError(errJson!(e)));

    let default_summary_stats = Some(SummaryStatistics {
        amount_total: 0,
        unpaid: 0,
        count: 0,
    });

    match agg {
        Err(_e) => {
            Ok(PayoutItemHistorySummaries {
                today: default_summary_stats.clone(),
                last_7_days: default_summary_stats.clone(),
                last_30_days: default_summary_stats.clone(),
                last_period: default_summary_stats.clone(),
                current_period: default_summary_stats.clone(),
                all_time: default_summary_stats,
            })
        }
        Ok(p) => {
            Ok(PayoutItemHistorySummaries {
                today: p.today.or(default_summary_stats.clone()),
                last_7_days: p.last_7_days.or(default_summary_stats.clone()),
                last_30_days: p.last_30_days.or(default_summary_stats.clone()),
                last_period: p.last_period.or(default_summary_stats.clone()),
                current_period: p.current_period.or(default_summary_stats.clone()),
                all_time: p.all_time.or(default_summary_stats),
            })
        }
    }

}

