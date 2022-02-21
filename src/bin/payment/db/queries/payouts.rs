use diesel::prelude::*;
use diesel::result::Error;
use diesel::PgConnection;
use diesel::sql_types::{ Text, Timestamp };

use gm::db;
// import traits
use chrono::Datelike;
use std::fmt::Write;
use itertools::{Itertools, Either};


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
    Payout,
    PayoutStatus,
    PayoutPeriod,
    PayeeType,
    PayoutAggregates,
    ConnectionQuery,
};
// use crate::models::paginate_page::*;
use crate::models::paginate_cursor::*;
use crate::models::payout_signatures::{
    SignedPayouts,
    PayoutApprovalType,
    PayoutIds,
    PaidPayouts,
};


////////////////////////
/// Payouts
////////////////////////


pub fn write_many_payouts(
    conn: &PgConnection,
    payouts_vec: &Vec<Payout>,
    // PayoutItems that are ready for a Payout
    payout_item_ids: &Vec<String>,
    // PayoutItems that are missing a payout method (paypal emails)
    missing_payout_method_ids: &Vec<String>,
    // PayoutItems that are refunds, and need to be deducted from subtotals
    refund_item_ids: &Vec<String>,
) -> Result<Vec<Payout>, DbError> {

    use db::schema::payouts;
    use db::schema::payout_items;


    conn.transaction::<Vec<Payout>, diesel::result::Error, _>(|| {
        // 1. Set payout_items status to PENDING_APPROVAL
        let _ = diesel::update(payout_items::table
            .filter(payout_items::id.eq_any(payout_item_ids)))
            .set(payout_items::payout_status.eq(PayoutStatus::PENDING_APPROVAL))
            .load::<PayoutItem>(conn);

        // 2. Set payout_items with missing payout metthods to MISSING_PAYOUT_METHOD
        let _ = diesel::update(payout_items::table
            .filter(payout_items::id.eq_any(missing_payout_method_ids)))
            .set(payout_items::payout_status.eq(PayoutStatus::MISSING_PAYOUT_METHOD))
            .load::<PayoutItem>(conn);

        // 3. Set payout_items refunds status to PENDING_REFUND
        let _ = diesel::update(payout_items::table
            .filter(payout_items::id.eq_any(refund_item_ids)))
            .set(payout_items::payout_status.eq(PayoutStatus::PENDING_REFUND))
            .load::<PayoutItem>(conn);

        // 4. Write payouts to db and return result
        let res = diesel::insert_into(payouts::table)
            .values(payouts_vec)
            .load::<Payout>(conn);

        // 5. Update payout_items.payout_id
        let update_query = conjure_batch_update_payout_items_query(payouts_vec);
        let _updated_pitems = diesel::sql_query(update_query)
            .load::<PayoutItem>(conn);

        res

    }).map_err(|e| DbError::PayoutWriteError(errJson!(e)))
}


pub fn conjure_batch_update_payout_items_query(payouts: &Vec<Payout>) -> String {

    let mut update_query = String::with_capacity(120 + 20 * payouts.len());
    update_query.push_str("UPDATE payout_items SET payout_id = temp.pid FROM (VALUES ");

    for p in payouts {
        for pitem_id in &p.payout_item_ids {
            let _ = write!(&mut update_query, "('{}', '{}'),", pitem_id, p.id);
        }
    }

    update_query.pop(); // remove trailing comma
    update_query.push_str(") AS temp(id, pid) WHERE temp.id = payout_items.id ");
    update_query.push_str("RETURNING *;");
    update_query
}
// UPDATE payout_items SET payout_id = temp.pid FROM (VALUES)
// AS temp(id, pid) WHERE temp.id = payout_items.id RETURNING *;


pub fn approve_many_payouts(
    conn: &PgConnection,
    approved_ids: PayoutIds,
    pending_ids: PayoutIds
) -> Result<PaidPayouts, DbError> {

    use db::schema::payouts;
    use db::schema::payout_items;

    //////////////////////////////////////////////////////
    /// Write Transaction
    //////////////////////////////////////////////////////
    conn.transaction::<PaidPayouts, diesel::result::Error, _>(|| {

        // 5a. Payouts with enough approvals
        // first set the payouts that can be paid out as PENDING_APPROVAL
        let approved = diesel::update(payouts::table
            .filter(
                payouts::id.eq_any(&approved_ids.payout_ids)
                .and(payouts::payout_status.eq(PayoutStatus::PENDING_APPROVAL))
            ))
            .set((
                payouts::payout_status.eq(PayoutStatus::PROCESSING),
                payouts::approved_by_ids.eq(&approved_ids.approver_ids)
            ))
            .load::<Payout>(conn);

        // do the same for payout items
        let _ = diesel::update(payout_items::table
            .filter(
                payout_items::id.eq_any(&approved_ids.payout_ids)
                .and(payout_items::payout_status.eq(PayoutStatus::PENDING_APPROVAL))
            ))
            .set(payout_items::payout_status.eq(PayoutStatus::PROCESSING))
            .load::<PayoutItem>(conn);


        // then set PENDING_REFUND items as REFUNDED
        let refund_approved = diesel::update(payouts::table
            .filter(
                payouts::id.eq_any(&approved_ids.payout_ids)
                .and(payouts::payout_status.eq(PayoutStatus::PENDING_REFUND))
            ))
            .set((
                payouts::payout_status.eq(PayoutStatus::REFUNDED),
                payouts::approved_by_ids.eq(&approved_ids.approver_ids)
            ))
            .load::<Payout>(conn);


        let _ = diesel::update(payout_items::table
            .filter(
                payout_items::id.eq_any(&approved_ids.pitem_ids)
                .and(payout_items::payout_status.eq(PayoutStatus::PENDING_REFUND))
            ))
            .set(payout_items::payout_status.eq(PayoutStatus::REFUNDED))
            .load::<PayoutItem>(conn);



        // 6a. Payouts which still need more approvals
        let pending = diesel::update(payouts::table
            .filter(payouts::id.eq_any(&pending_ids.payout_ids)))
            .set((
                payouts::payout_status.eq(PayoutStatus::PENDING_APPROVAL),
                payouts::approved_by_ids.eq(&pending_ids.approver_ids)
            ))
            .load::<Payout>(conn);

        let _ = diesel::update(payout_items::table
            .filter(payout_items::id.eq_any(pending_ids.pitem_ids)))
            .set(payout_items::payout_status.eq(PayoutStatus::PENDING_APPROVAL))
            .load::<PayoutItem>(conn);


        match (approved, refund_approved, pending) {
            (Err(e1), _, _) => Err(e1),
            (_, Err(e2), _) => Err(e2),
            (_, _, Err(e3)) => Err(e3),
            (Ok(a), Ok(r), Ok(p)) => Ok(PaidPayouts::new(a, r, p)),
        }

    }).map_err(|e| DbError::PayoutWriteError(errJson!(e)))

}


/// For deleting testing data only
pub fn delete_payouts(
    conn: &PgConnection,
    payout_ids: &Vec<String>,
) -> Result<Vec<Payout>, DbError> {

    use db::schema::payouts;

    diesel::delete(payouts::table
        .filter(payouts::id.eq_any(payout_ids)))
        .load::<Payout>(conn)
        .map_err(|e| DbError::PayoutWriteError(errJson!(e)))
}




pub fn read_many_payouts(
    conn: &PgConnection,
    payout_ids: &Vec<String>,
) -> Result<Vec<Payout>, DbError> {

    use db::schema::payouts;
    use diesel::dsl::*;

    payouts::table
        .filter(payouts::id.eq_any(payout_ids))
        .load::<Payout>(conn)
        .map_err(|e| DbError::PayoutReadError(errJson!(e)))
}


pub fn read_many_payouts_by_payee_id_paginated(
    conn: &PgConnection,
    payee_id: &str,
    connection_query: ConnectionQuery,
) -> Result<(Vec<Payout>, i64, bool), DbError> {

    use db::schema::payouts;
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

    match (maybe_cursor, lessThan) {
        (Some(cursor), true) => {
            payouts::table
                .select(payouts::all_columns)
                .filter(
                    payouts::payee_id.eq(payee_id)
                    // cursor: lessThan
                    .and(payouts::created_at.lt(cursor.value))
                )
                .paginate_by_cursor(
                    orderField,
                    Some(queryAscending),
                    connection_query.pageBackwards,
                )
                .per_page(connection_query.count)
                .load_and_count_pages::<Payout>(conn)
                .map_err(|e| DbError::PayoutReadError(errJson!(e)))
        },
        (Some(cursor), false) => {
            payouts::table
                .select(payouts::all_columns)
                .filter(
                    payouts::payee_id.eq(payee_id)
                    // cursor: greaterThan
                    .and(payouts::created_at.gt(cursor.value))
                )
                .paginate_by_cursor(
                    orderField,
                    Some(queryAscending),
                    connection_query.pageBackwards,
                )
                .per_page(connection_query.count)
                .load_and_count_pages::<Payout>(conn)
                .map_err(|e| DbError::PayoutReadError(errJson!(e)))
        },
        (None, _) => {
            payouts::table
                .select(payouts::all_columns)
                .filter(
                    payouts::payee_id.eq(payee_id)
                )
                .paginate_by_cursor(
                    orderField,
                    Some(queryAscending),
                    connection_query.pageBackwards,
                )
                .per_page(connection_query.count)
                .load_and_count_pages::<Payout>(conn)
                .map_err(|e| DbError::PayoutReadError(errJson!(e)))
        },
    }
}



pub fn read_many_payouts_in_period_paginated(
    conn: &PgConnection,
    start_date: chrono::NaiveDateTime,
    end_date: chrono::NaiveDateTime,
    status: Option<PayoutStatus>,
    connection_query: ConnectionQuery,
) -> Result<(Vec<Payout>, i64, bool), DbError> {

    use db::schema::payouts;
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
            payouts::table
                .select(payouts::all_columns)
                .filter(
                    payouts::start_period.eq(start_date)
                    .and(payouts::end_period.eq(end_date))
                    .and(payouts::payout_status.eq(payout_status))
                    // cursor: lessThan
                    .and(payouts::created_at.lt(cursor.value))
                )
                .paginate_by_cursor(
                    orderField,
                    Some(queryAscending),
                    connection_query.pageBackwards,
                )
                .per_page(connection_query.count)
                .load_and_count_pages::<Payout>(conn)
                .map_err(|e| DbError::PayoutReadError(errJson!(e)))
        },
        (Some(payout_status), Some(cursor), false) => {
            payouts::table
                .select(payouts::all_columns)
                .filter(
                    payouts::start_period.eq(start_date)
                    .and(payouts::end_period.eq(end_date))
                    .and(payouts::payout_status.eq(payout_status))
                    // cursor: greaterThan
                    .and(payouts::created_at.gt(cursor.value))
                )
                .paginate_by_cursor(
                    orderField,
                    Some(queryAscending),
                    connection_query.pageBackwards,
                )
                .per_page(connection_query.count)
                .load_and_count_pages::<Payout>(conn)
                .map_err(|e| DbError::PayoutReadError(errJson!(e)))
        },
        (None, Some(cursor), true) => {
            payouts::table
                .select(payouts::all_columns)
                .filter(
                    payouts::start_period.eq(start_date)
                    .and(payouts::end_period.eq(end_date))
                    // cursor: lessThan
                    .and(payouts::created_at.lt(cursor.value))
                )
                .paginate_by_cursor(
                    orderField,
                    Some(queryAscending),
                    connection_query.pageBackwards,
                )
                .per_page(connection_query.count)
                .load_and_count_pages::<Payout>(conn)
                .map_err(|e| DbError::PayoutReadError(errJson!(e)))
        },
        (None, Some(cursor), false) => {
            payouts::table
                .select(payouts::all_columns)
                .filter(
                    payouts::start_period.eq(start_date)
                    .and(payouts::end_period.eq(end_date))
                    // cursor: greaterThan
                    .and(payouts::created_at.gt(cursor.value))
                )
                .paginate_by_cursor(
                    orderField,
                    Some(queryAscending),
                    connection_query.pageBackwards,
                )
                .per_page(connection_query.count)
                .load_and_count_pages::<Payout>(conn)
                .map_err(|e| DbError::PayoutReadError(errJson!(e)))
        },
        (Some(payout_status), None, _) => {
            payouts::table
                .select(payouts::all_columns)
                .filter(
                    payouts::start_period.eq(start_date)
                    .and(payouts::end_period.eq(end_date))
                    .and(payouts::payout_status.eq(payout_status))
                )
                .paginate_by_cursor(
                    orderField,
                    Some(queryAscending),
                    connection_query.pageBackwards,
                )
                .per_page(connection_query.count)
                .load_and_count_pages::<Payout>(conn)
                .map_err(|e| DbError::PayoutReadError(errJson!(e)))
        },
        (None, None, _) => {
            payouts::table
                .select(payouts::all_columns)
                .filter(
                    payouts::start_period.eq(start_date)
                    .and(payouts::end_period.eq(end_date))
                )
                .paginate_by_cursor(
                    orderField,
                    Some(queryAscending),
                    connection_query.pageBackwards,
                )
                .per_page(connection_query.count)
                .load_and_count_pages::<Payout>(conn)
                .map_err(|e| DbError::PayoutReadError(errJson!(e)))
        },
    }
}

pub fn read_payouts_for_payee_id_in_period(
    conn: &PgConnection,
    payee_id: &str,
    start_date: chrono::NaiveDateTime,
    end_date: chrono::NaiveDateTime,
) -> Result<Vec<Payout>, DbError> {

    use db::schema::payouts;
    use diesel::dsl::*;

    payouts::table
        .select(payouts::all_columns)
        .filter(
            payouts::payee_id.eq(payee_id)
            .and(payouts::start_period.eq(start_date))
            .and(payouts::end_period.eq(end_date))
        )
        .load::<Payout>(conn)
        .map_err(|e| DbError::PayoutReadError(errJson!(e)))
}


pub fn update_payouts_post_paypal_payout(
    conn: &PgConnection,
    payout_ids: &Vec<String>,
    refunding_ids: &Vec<String>,
    paypal_payout_batch_id: String,
) -> Result<Vec<Payout>, DbError> {

    use db::schema::payouts;
    use db::schema::payout_items;

    conn.transaction::<Vec<Payout>, diesel::result::Error, _>(|| {

        // 1. Set payout_items associated with PayoutIds' status to PAID
        let _ = diesel::update(
                payout_items::table
                .filter(
                    payout_items::payout_id.eq_any(payout_ids)
                    .and(payout_items::payout_status.eq(PayoutStatus::PENDING_APPROVAL))
                )
            )
            .set(payout_items::payout_status.eq(PayoutStatus::PAID))
            .load::<PayoutItem>(conn);

        // 2. Set payout_items associated with refunded items status to REFUNDED
        let _ = diesel::update(
                payout_items::table
                .filter(
                    payout_items::payout_id.eq_any(refunding_ids)
                    .and(payout_items::payout_status.eq(PayoutStatus::PENDING_REFUND))
                )
            )
            .set(payout_items::payout_status.eq(PayoutStatus::REFUNDED))
            .load::<PayoutItem>(conn);

        // 3. Set payout_items associated with Platform to RETAINED
        // The ones which have bee gropu into a payout already
        let _ = diesel::update(
                payout_items::table
                .filter(
                    payout_items::payee_type.eq(PayeeType::PLATFORM)
                    .and(payout_items::payout_status.eq_any(vec![
                        PayoutStatus::MISSING_PAYOUT_METHOD,
                        PayoutStatus::PENDING_REFUND
                    ]))
                )
            )
            .set(payout_items::payout_status.eq(PayoutStatus::RETAINED))
            .load::<PayoutItem>(conn);

        // 4. Update payouts with payout (Paypal BatchID) response and return result
        diesel::update(payouts::table
            .filter(payouts::id.eq_any(payout_ids)))
            .set((
                payouts::payout_status.eq(PayoutStatus::PAID),
                payouts::payout_batch_id.eq(&paypal_payout_batch_id),
            ))
            .load::<Payout>(conn)

    }).map_err(|e| DbError::PayoutWriteError(errJson!(e)))
}



pub fn read_payout_aggregates(
    conn: &PgConnection,
    start_date: chrono::NaiveDateTime,
    end_date: chrono::NaiveDateTime,
    sort_ascending: bool,
) -> PayoutAggregates {

    use db::schema::payouts;
    use diesel::dsl::*;

    let ascDesc = if sort_ascending { "ASC" } else { "DESC" };

    debug!("start date: {:?}", start_date);
    debug!("end date: {:?}", end_date);

    // Sum over transactions.subtotal
    let agg = diesel::sql_query(format!(r#"
        SELECT
            created_at,
            payout_date,
            SUM(amount) OVER () as amount_total,
            COUNT(*) OVER ()
        FROM (
            SELECT * FROM payouts
        ) q
        WHERE payout_date > $1 AND payout_date < $2
        ORDER BY created_at {}
        LIMIT 1
    "#, ascDesc))
    .bind::<Timestamp, _>(start_date)
    .bind::<Timestamp, _>(end_date)
    .get_result::<PayoutAggregates>(conn)
    .map_err(|e| DbError::PayoutReadError(errJson!(e)));

    debug!("read_payout_aggregates db result: {:?}", agg);

    match agg {
        Ok(res) => res,
        Err(_e) => PayoutAggregates::new(),
    }
}


pub fn read_payout_aggregates_by_store_id(
    conn: &PgConnection,
    store_id: &str,
    sort_ascending: bool,
) -> PayoutAggregates {

    use db::schema::payouts;
    use diesel::dsl::*;

    let ascDesc = if sort_ascending { "ASC" } else { "DESC" };

    // Sum over transactions.subtotal
    let agg = diesel::sql_query(format!(r#"
        SELECT
            created_at,
            payout_date,
            SUM(amount) OVER () as amount_total,
            COUNT(*) OVER ()
        FROM (
            SELECT * FROM payouts
        ) q
        WHERE payee_id = $1
        ORDER BY created_at {}
        LIMIT 1
    "#, ascDesc))
    .bind::<Text, _>(store_id)
    .get_result::<PayoutAggregates>(conn)
    .map_err(|e| DbError::PayoutReadError(errJson!(e)));

    match agg {
        Ok(res) => res,
        Err(_e) => PayoutAggregates::new(),
    }
}