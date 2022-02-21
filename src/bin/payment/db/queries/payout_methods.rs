use diesel::prelude::*;
use diesel::result::Error;
use diesel::PgConnection;
use gm::db;

use crate::models::{
    PayoutMethod,
    ErrJson,
    DbError,
};


////////////////////////
/// Payout Methods
////////////////////////


pub fn write_payout_method(
    conn: &PgConnection,
    payout_method: PayoutMethod,
) -> Result<PayoutMethod, DbError> {

    use db::schema::payout_methods;

    diesel::insert_into(payout_methods::table)
        .values(payout_method)
        .get_result::<PayoutMethod>(conn)
        .map_err(|e| DbError::PayoutMethodWriteError(errJson!(e)))
}

pub fn read_payout_method(
    conn: &PgConnection,
    payout_method_id: String,
) -> Result<PayoutMethod, DbError> {

    use db::schema::payout_methods;

    payout_methods::table
        .filter(payout_methods::id.eq(payout_method_id))
        .get_result::<PayoutMethod>(conn)
        .map_err(|e| DbError::PayoutMethodReadError(errJson!(e)))
}


pub fn read_payout_methods_by_ids(
    conn: &PgConnection,
    payout_method_ids: Vec<String>,
) -> Result<Vec<PayoutMethod>, DbError> {

    use db::schema::payout_methods;

    payout_methods::table
        .filter(payout_methods::id.eq_any(payout_method_ids))
        .load::<PayoutMethod>(conn)
        .map_err(|e| DbError::PayoutMethodReadError(errJson!(e)))
}


pub fn read_payout_methods_by_payee_ids(
    conn: &PgConnection,
    // store_id, or affiliate_ids are both payee_ids
    payee_ids: &Vec<String>,
) -> Result<Vec<PayoutMethod>, DbError> {

    use db::schema::payout_methods;

    payout_methods::table
        .filter(payout_methods::payee_id.eq_any(payee_ids))
        .load::<PayoutMethod>(conn)
        .map_err(|e| DbError::PayoutMethodReadError(errJson!(e)))
}


pub fn insert_payout_method_by_payee_id(
    conn: &PgConnection,
    // store_id, or user_ids are both payee_ids
    payout_method: PayoutMethod,
) -> Result<PayoutMethod, DbError> {

    use db::schema::payout_methods;

    conn.transaction::<PayoutMethod, diesel::result::Error, _>(|| {

        diesel::insert_into(payout_methods::table)
            .values(payout_method)
            .get_result::<PayoutMethod>(conn)

    }).map_err(|e| DbError::PayoutWriteError(errJson!(e)))

}



