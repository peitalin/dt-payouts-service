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
    CartDb,
    PaymentMethodDb,
    PaymentMethodAddress,
    Refund,
    PayoutItem,
    Payout,
    ConnectionQuery,
};


////////////////////////
/// Payment Methods
////////////////////////


pub fn write_payment_method(
    conn: &PgConnection,
    payment_method: PaymentMethodDb,
) -> Result<PaymentMethodDb, DbError> {

    use db::schema::payment_methods;

    diesel::insert_into(payment_methods::table)
        .values(payment_method)
        .get_result::<PaymentMethodDb>(conn)
        .map_err(|e| DbError::PaymentMethodWriteError(errJson!(e)))
}

pub fn read_payment_method(
    conn: &PgConnection,
    payment_method_id: String,
) -> Result<PaymentMethodDb, DbError> {

    use db::schema::payment_methods;

    payment_methods::table
        .filter(payment_methods::id.eq(payment_method_id))
        .get_result::<PaymentMethodDb>(conn)
        .map_err(|e| DbError::PaymentMethodReadError(errJson!(e)))
}

pub fn read_many_payment_methods(
    conn: &PgConnection,
    payment_method_ids: Vec<String>,
) -> Result<Vec<PaymentMethodDb>, DbError> {

    use db::schema::payment_methods;
    // use diesel::expression::any;
    use diesel::dsl::*;

    // methods for returning multiple rows
    // https://docs.diesel.rs/diesel/query_dsl/trait.RunQueryDsl.html
    payment_methods::table
        .filter(payment_methods::id.eq_any(payment_method_ids))
        .load::<PaymentMethodDb>(conn)
        .map_err(|e| DbError::PaymentMethodReadError(errJson!(e)))
}

pub fn read_payment_methods_by_user_id(
    conn: &PgConnection,
    user_id: &String,
) -> Result<Vec<PaymentMethodDb>, DbError> {

    use db::schema::payment_methods;
    // use diesel::expression::any;
    use diesel::dsl::*;

    // methods for returning multiple rows
    // https://docs.diesel.rs/diesel/query_dsl/trait.RunQueryDsl.html
    payment_methods::table
        .filter(payment_methods::user_id.eq(user_id))
        .load::<PaymentMethodDb>(conn)
        .map_err(|e| DbError::PaymentMethodReadError(errJson!(e)))
}


pub fn write_payment_method_address(
    conn: &PgConnection,
    payment_method_address: PaymentMethodAddress,
) -> Result<PaymentMethodAddress, DbError> {

    use db::schema::payment_method_addresses;

    diesel::insert_into(payment_method_addresses::table)
        .values(payment_method_address)
        .get_result::<PaymentMethodAddress>(conn)
        .map_err(|e| DbError::PaymentMethodAddressWriteError(errJson!(e)))
}

pub fn read_payment_method_address(
    conn: &PgConnection,
    payment_method_id: String,
) -> Result<PaymentMethodAddress, DbError> {

    use db::schema::payment_method_addresses;

    payment_method_addresses::table
        .filter(payment_method_addresses::payment_method_id.eq(payment_method_id))
        .get_result::<PaymentMethodAddress>(conn)
        .map_err(|e| DbError::PaymentMethodAddressReadError(errJson!(e)))
}


pub fn read_payment_method_id_for_order(
    conn: &PgConnection,
    order_id: &str,
) -> (Option<String>, Option<String>) {

    use db::schema::transactions;

    // get 1 result, each order may have multiple refunds with same order_id
    // all of them should have an associated payment_method_id
    let tx_row = transactions::table
                    .filter(transactions::order_id.eq(order_id))
                    .get_result::<Transaction>(conn)
                    .map_err(|e| DbError::TransactionReadError(errJson!(e)));

    match tx_row {
        Err(_e) => (None, None),
        Ok(tx_row) => (tx_row.payment_method_id, tx_row.customer_id)
    }
}


pub fn delete_payment_method(
    conn: &PgConnection,
    payment_method_id: &String,
    user_id: &String,
) -> Result<Vec<PaymentMethodDb>, DbError> {

    use db::schema::payment_methods;
    use diesel::dsl::*;


    // attempt to delete payment
    let _ = diesel::delete(payment_methods::table
                .filter(payment_methods::id.eq(payment_method_id)))
                .get_result::<PaymentMethodDb>(conn)
                .map_err(|e| DbError::PaymentMethodWriteError(errJson!(e)));

    read_payment_methods_by_user_id(conn, user_id)
}
