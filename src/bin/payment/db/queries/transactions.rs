use diesel::prelude::*;
use diesel::result::Error;
use diesel::PgConnection;
// from ./src/db
use gm::db;

use crate::models::{
    StripeError,
    DbError,
    ErrJson,
    Transaction,
    TransactionAggregates,
    CartDb,
    PaymentMethodDb,
    PaymentMethodAddress,
    Refund,
    PayoutItem,
    Payout,
    ConnectionQuery,
};
use crate::models::paginate_page::PaginatedPage;
use crate::models::paginate_page::PaginatePage;
use crate::models::paginate_cursor::*;


/////////////////////
/// Transactions
/////////////////////


pub fn read_transactions_paginate_page(
    conn: &PgConnection,
    count_per_page: i64,
) -> Result<(Vec<Transaction>, i64), DbError> {

    use db::schema::transactions;
    use diesel::dsl::*;
    // paginate must come after .select()
    // select() implements Query, etc
    transactions::table
        .select(transactions::all_columns)
        .paginate_by_page(1)
        .per_page(count_per_page)
        .load_and_count_pages::<Transaction>(conn)
        .map_err(|e| DbError::TransactionReadError(errJson!(e)))

}

pub fn read_transactions_paginate_cursor(
    conn: &PgConnection,
    start_date: chrono::NaiveDateTime,
    end_date: chrono::NaiveDateTime,
    connection_query: ConnectionQuery,
) -> Result<(Vec<Transaction>, i64, bool), DbError> {

    use db::schema::transactions;
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
            transactions::table
                .select(transactions::all_columns)
                .filter(
                    transactions::created_at.gt(start_date)
                    .and(transactions::created_at.lt(end_date))
                    .and(transactions::created_at.lt(cursor.value))
                )
                .paginate_by_cursor(
                    orderField,
                    Some(queryAscending),
                    connection_query.pageBackwards,
                )
                .per_page(connection_query.count)
                .load_and_count_pages::<Transaction>(conn)
                .map_err(|e| DbError::TransactionReadError(errJson!(e)))
        },
        (Some(cursor), false) => {
            transactions::table
                .select(transactions::all_columns)
                .filter(
                    transactions::created_at.gt(start_date)
                    .and(transactions::created_at.lt(end_date))
                    .and(transactions::created_at.gt(cursor.value))
                )
                .paginate_by_cursor(
                    orderField,
                    Some(queryAscending),
                    connection_query.pageBackwards,
                )
                .per_page(connection_query.count)
                .load_and_count_pages::<Transaction>(conn)
                .map_err(|e| DbError::TransactionReadError(errJson!(e)))
        },
        (None, _) => {
            transactions::table
                .select(transactions::all_columns)
                .filter(
                    transactions::created_at.gt(start_date)
                    .and(transactions::created_at.lt(end_date))
                )
                .paginate_by_cursor(
                    orderField,
                    Some(queryAscending),
                    connection_query.pageBackwards,
                )
                .per_page(connection_query.count)
                .load_and_count_pages::<Transaction>(conn)
                .map_err(|e| DbError::TransactionReadError(errJson!(e)))
        },
    }
}

pub fn write_transaction(
    conn: &PgConnection,
    tx: &Transaction,
) -> Result<Transaction, DbError> {

    use db::schema::transactions;

    diesel::insert_into(transactions::table)
        .values(tx)
        .get_result::<Transaction>(conn)
        .map_err(|e| DbError::TransactionWriteError(errJson!(e)))
}

pub fn write_transactions(
    conn: &PgConnection,
    transactions: &Vec<Transaction>,
) -> Result<Vec<Transaction>, DbError> {

    use db::schema::transactions;

    diesel::insert_into(transactions::table)
        .values(transactions)
        .load::<Transaction>(conn)
        .map_err(|e| DbError::TransactionWriteError(errJson!(e)))
}

pub fn delete_transactions(
    conn: &PgConnection,
    tx_ids: &Vec<String>,
) -> Result<Vec<Transaction>, DbError> {

    use db::schema::transactions;

    diesel::delete(transactions::table
        .filter(transactions::id.eq_any(tx_ids)))
        .load::<Transaction>(conn)
        .map_err(|e| DbError::TransactionWriteError(errJson!(e)))
}

pub fn read_many_transactions_by_ids(
    conn: &PgConnection,
    transaction_ids: Vec<String>,
) -> Result<Vec<Transaction>, DbError> {

    use db::schema::transactions;
    use diesel::dsl::*;

    transactions::table
        .filter(transactions::id.eq_any(transaction_ids))
        .load::<Transaction>(conn)
        .map_err(|e| DbError::TransactionReadError(errJson!(e)))
}

pub fn read_recent_transactions(
    conn: &PgConnection,
    limit_count: i64,
) -> Result<Vec<Transaction>, DbError> {

    use db::schema::transactions;
    use diesel::dsl::*;

    transactions::table
        .order(transactions::created_at.desc())
        .limit(limit_count)
        .load::<Transaction>(conn)
        .map_err(|e| DbError::TransactionReadError(errJson!(e)))
}


pub fn write_transaction_and_refund_and_refund_items(
    conn: &PgConnection,
    tx: &Transaction,
    refund: &Refund,
    refund_items: &Vec<PayoutItem>,
) -> Result<(Transaction, Refund, Vec<PayoutItem>), DbError> {

    use db::schema::transactions;
    use db::schema::refunds;
    use db::schema::payout_items;

    conn.transaction::<(Transaction, Refund, Vec<PayoutItem>), diesel::result::Error, _>(|| {

        let tx_result = diesel::insert_into(transactions::table)
            .values(tx)
            .get_result::<Transaction>(conn);

        let refund_result = diesel::insert_into(refunds::table)
            .values(refund)
            .get_result::<Refund>(conn);

        let refund_items_result = diesel::insert_into(payout_items::table)
            .values(refund_items)
            .load::<PayoutItem>(conn);

        match (tx_result, refund_result, refund_items_result) {
            (Ok(t), Ok(r), Ok(p)) => Ok((t, r, p)),
            (Err(e1), _, _) => Err(e1),
            (_, Err(e2), _) => Err(e2),
            (_, _, Err(e3)) => Err(e3),
        }

    }).map_err(|e| DbError::TransactionWriteError(errJson!(e)))
}


pub fn write_transaction_and_payout_items(
    conn: &PgConnection,
    tx: &Transaction,
    payout_items: &Vec<PayoutItem>,
) -> Result<(Transaction, Vec<PayoutItem>), DbError> {

    use db::schema::transactions;
    use db::schema::payout_items;

    conn.transaction::<(Transaction, Vec<PayoutItem>), diesel::result::Error, _>(|| {

        let tx_result = diesel::insert_into(transactions::table)
            .values(tx)
            .get_result::<Transaction>(conn);

        let pitem_result = diesel::insert_into(payout_items::table)
            .values(payout_items)
            .load::<PayoutItem>(conn);

        match (tx_result, pitem_result) {
            (Ok(t), Ok(p)) => Ok((t, p)),
            (Err(e1), _) => Err(e1),
            (_, Err(e2)) => Err(e2),
        }

    }).map_err(|e| DbError::PayoutItemWriteError(errJson!(e)))
}

pub fn read_transaction_aggregates(
    conn: &PgConnection,
    start_date: chrono::NaiveDateTime,
    end_date: chrono::NaiveDateTime,
    sort_ascending: bool,
) -> TransactionAggregates {

    use db::schema::transactions;
    use diesel::dsl::*;

    let ascDesc = if sort_ascending { "ASC" } else { "DESC" };

    // Sum over transactions.subtotal
    let agg = diesel::sql_query(format!(r#"
        SELECT
            created_at,
            SUM(subtotal) OVER () as subtotal_sum,
            SUM(payment_processing_fee) OVER () as fees_total,
            COUNT(*) OVER ()
        FROM (
            SELECT * FROM transactions
        ) q
        WHERE created_at > '{}' AND created_at < '{}'
        ORDER BY created_at {}
        LIMIT 1
    "#, start_date, end_date, ascDesc))
    .get_result::<TransactionAggregates>(conn)
    .map_err(|e| DbError::TransactionReadError(errJson!(e)));

    match agg {
        Ok(res) => res,
        Err(_e) => TransactionAggregates::new(),
    }
}
