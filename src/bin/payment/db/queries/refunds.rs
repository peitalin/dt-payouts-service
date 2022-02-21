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
use crate::models::paginate_page::*;
use crate::models::paginate_cursor::*;


////////////////////////
/// Refunds
////////////////////////


pub fn write_refund(
    conn: &PgConnection,
    refund: Refund,
) -> Result<Refund, DbError> {

    use db::schema::refunds;

    diesel::insert_into(refunds::table)
        .values(refund)
        .get_result::<Refund>(conn)
        .map_err(|e| DbError::RefundWriteError(errJson!(e)))
}

pub fn read_refund(
    conn: &PgConnection,
    refund: Refund,
) -> Result<Refund, DbError> {

    use db::schema::refunds;

    refunds::table
        .filter(refunds::id.eq(refund.id))
        .get_result::<Refund>(conn)
        .map_err(|e| DbError::RefundReadError(errJson!(e)))
}

pub fn read_many_refunds(
    conn: &PgConnection,
    refund_ids: Vec<String>,
) -> Result<Vec<Refund>, DbError> {

    use db::schema::refunds;
    use diesel::dsl::*;

    refunds::table
        .filter(refunds::id.eq_any(refund_ids))
        .load::<Refund>(conn)
        .map_err(|e| DbError::RefundReadError(errJson!(e)))
}