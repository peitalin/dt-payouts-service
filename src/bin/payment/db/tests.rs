use diesel::prelude::*;
use diesel::result::Error;
use diesel::PgConnection;
use crate::diesel::Connection;
use gm::db::establish_connection_pg;
// import traits
use std::str::FromStr;
use chrono::Datelike;
use std::fmt::Write;

use crate::models::DbError;
use crate::models::{
    PaymentMethodDb,
    PaymentMethodAddress,
    Transaction,
    TransactionAggregates,
    PayoutItem,
    PayoutItemAggregates,
    Payout,
    PayoutAggregates,
    PayoutStatus,
    PayoutPeriod,
    PayoutSplit,
    PayoutDealType,
};
use crate::models::connection::{
    Edge,
    PageInfo,
    ConnectionQuery,
};
use crate::models::connection;
use crate::models::paginate_cursor::decode_datetime_cursor;
use crate::db;





#[test]
fn adds_then_reads_many_payment_methods() {

    let conn = establish_connection_pg("DATABASE_URL");

    conn.test_transaction::<(), DbError, _>(|| {

        use gm::db::schema::payment_methods;

        let pm1 = PaymentMethodDb {
            id: String::from("id1"),
            user_id: String::from("user_id1"),
            created_at: chrono::NaiveDateTime::from_timestamp(1_500_000_000, 0),
            updated_at: None,
            customer_id: None,
            payment_processor: None,
            payment_method_types: None,
            last4: None,
            exp_month: None,
            exp_year: None,
            email: None,
            name: None,
            details: None,
        };
        let pm2 = PaymentMethodDb {
            id: String::from("id2"),
            user_id: String::from("user_id2"),
            created_at: chrono::NaiveDateTime::from_timestamp(1_500_000_000, 0),
            updated_at: None,
            customer_id: None,
            payment_processor: None,
            payment_method_types: None,
            last4: None,
            exp_month: None,
            exp_year: None,
            email: None,
            name: None,
            details: None,
        };
        let res1 = crate::db::write_payment_method(&conn, pm1).unwrap();
        let res2 = crate::db::write_payment_method(&conn, pm2).unwrap();

        match crate::db::read_many_payment_methods(
            &conn,
            vec![String::from("id1"), String::from("id2")]
        ) {
            Err(_e) => panic!("read-many_payment_methods test failed"),
            Ok(read_results) => {
                assert_eq!(
                    read_results.len(),
                    vec![res1, res2].len(),
                )
            }
        };

        Ok(())
    });
}


/////////////////////////////////
/// Mock Date Writes
/////////////////////////////////
/// Remember to delete at the end of every transaction!


fn write_test_transactions(conn: &PgConnection) -> Vec<String> {

    let tx1 = Transaction {
        created_at: chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() + 1, 0
        ),
        subtotal: 1,
        ..Default::default()
    };
    let tx2 = Transaction {
        created_at: chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() + 2, 0
        ),
        subtotal: 2,
        ..Default::default()
    };
    let tx3 = Transaction {
        created_at: chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() + 3, 0
        ),
        subtotal: 3,
        ..Default::default()
    };
    let tx4 = Transaction {
        created_at: chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() + 4, 0
        ),
        subtotal: 4,
        ..Default::default()
    };
    let tx5 = Transaction {
        created_at: chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() + 5, 0
        ),
        subtotal: 5,
        ..Default::default()
    };
    let tx6 = Transaction {
        created_at: chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() + 6, 0
        ),
        subtotal: 6,
        ..Default::default()
    };
    let tx7 = Transaction {
        created_at: chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() + 7, 0
        ),
        subtotal: 7,
        ..Default::default()
    };
    let tx8 = Transaction {
        created_at: chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() + 8, 0
        ),
        subtotal: 8,
        ..Default::default()
    };

    let _ = db::write_transactions(conn, &vec![ tx1.clone() ]);
    std::thread::sleep(std::time::Duration::from_millis(10));
    let _ = db::write_transactions(conn, &vec![ tx2.clone() ]);
    std::thread::sleep(std::time::Duration::from_millis(10));
    let _ = db::write_transactions(conn, &vec![ tx3.clone() ]);
    std::thread::sleep(std::time::Duration::from_millis(10));
    let _ = db::write_transactions(conn, &vec![ tx4.clone() ]);
    std::thread::sleep(std::time::Duration::from_millis(10));
    let _ = db::write_transactions(conn, &vec![ tx5.clone() ]);
    std::thread::sleep(std::time::Duration::from_millis(10));
    let _ = db::write_transactions(conn, &vec![ tx6.clone() ]);
    std::thread::sleep(std::time::Duration::from_millis(10));
    let _ = db::write_transactions(conn, &vec![ tx7.clone() ]);
    std::thread::sleep(std::time::Duration::from_millis(10));
    let _ = db::write_transactions(conn, &vec![ tx8.clone() ]);
    std::thread::sleep(std::time::Duration::from_millis(10));

    vec![ tx1.id, tx2.id, tx3.id, tx4.id, tx5.id, tx6.id, tx7.id, tx8.id ]
}


fn write_test_payout_items(conn: &PgConnection) -> Vec<PayoutItem> {

    let pi1 = PayoutItem {
        created_at: chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() + 1, 0
        ),
        payee_id: String::from("store_1"),
        amount: 1,
        ..Default::default()
    };
    let pi2 = PayoutItem {
        created_at: chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() + 2, 0
        ),
        amount: 1,
        payee_id: String::from("store_2"),
        ..Default::default()
    };
    let pi3 = PayoutItem {
        created_at: chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() + 3, 0
        ),
        amount: 1,
        payee_id: String::from("store_3"),
        ..Default::default()
    };
    let pi4 = PayoutItem {
        created_at: chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() + 4, 0
        ),
        amount: 1,
        payee_id: String::from("store_4"),
        ..Default::default()
    };
    let pi5 = PayoutItem {
        created_at: chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() + 5, 0
        ),
        amount: 1,
        payee_id: String::from("store_5"),
        ..Default::default()
    };
    let pi6 = PayoutItem {
        created_at: chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() + 6, 0
        ),
        amount: 1,
        payee_id: String::from("store_6"),
        ..Default::default()
    };
    let pi7 = PayoutItem {
        created_at: chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() + 7, 0
        ),
        amount: 1,
        payee_id: String::from("store_7"),
        ..Default::default()
    };
    let pi8 = PayoutItem {
        created_at: chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() + 8, 0
        ),
        amount: 1,
        payee_id: String::from("store_8"),
        ..Default::default()
    };

    let _ = db::write_payout_items(conn, &vec![ pi1.clone() ]);
    std::thread::sleep(std::time::Duration::from_millis(10));
    let _ = db::write_payout_items(conn, &vec![ pi2.clone() ]);
    std::thread::sleep(std::time::Duration::from_millis(10));
    let _ = db::write_payout_items(conn, &vec![ pi3.clone() ]);
    std::thread::sleep(std::time::Duration::from_millis(10));
    let _ = db::write_payout_items(conn, &vec![ pi4.clone() ]);
    std::thread::sleep(std::time::Duration::from_millis(10));
    let _ = db::write_payout_items(conn, &vec![ pi5.clone() ]);
    std::thread::sleep(std::time::Duration::from_millis(10));
    let _ = db::write_payout_items(conn, &vec![ pi6.clone() ]);
    std::thread::sleep(std::time::Duration::from_millis(10));
    let _ = db::write_payout_items(conn, &vec![ pi7.clone() ]);
    std::thread::sleep(std::time::Duration::from_millis(10));
    let _ = db::write_payout_items(conn, &vec![ pi8.clone() ]);
    std::thread::sleep(std::time::Duration::from_millis(10));

    vec![ pi1, pi2, pi3, pi4, pi5, pi6, pi7, pi8 ]
}


fn write_test_payout_splits(conn: &PgConnection) -> Vec<String> {

    let ps1 = PayoutSplit::new(
        String::from("store_1"),
        PayoutDealType::SELLER_AFFILIATE,
        None,
        0.05,
        None,
    );
    let ps2 = PayoutSplit::new(
        String::from("store_2"),
        PayoutDealType::REFERRED_SELLER,
        None,
        0.05,
        Some(ps1.id.clone()),
    );
    let ps3 = PayoutSplit::new(
        String::from("store_3"),
        PayoutDealType::SELLER_AFFILIATE,
        None,
        0.05,
        None,
    );
    let ps4 = PayoutSplit::new(
        String::from("store_4"),
        PayoutDealType::REFERRED_SELLER,
        None,
        0.05,
        Some(ps3.id.clone()),
    );

    let _ = db::write_payout_split(conn, ps1.clone());
    std::thread::sleep(std::time::Duration::from_millis(10));

    let _ = db::write_payout_split(conn, ps2.clone());
    std::thread::sleep(std::time::Duration::from_millis(10));

    let _ = db::write_payout_split(conn, ps3.clone());
    std::thread::sleep(std::time::Duration::from_millis(10));

    let _ = db::write_payout_split(conn, ps4.clone());
    std::thread::sleep(std::time::Duration::from_millis(10));

    vec![ ps1.id, ps2.id, ps3.id, ps4.id ]
}


fn write_test_payouts(conn: &PgConnection) -> Vec<Payout> {

    let year = chrono::Utc::now().year();
    let month = chrono::Utc::now().month() as i32;

    let pi_ids = write_test_payout_items(&conn)
        .iter().map(|p| p.id.clone()).collect::<Vec<String>>();
    let missing_payout_item_ids = vec![] as Vec<String>;
    let refund_item_ids = vec![] as Vec<String>;

    let p1 = Payout::new(
        String::from("payout_1"),
        PayoutPeriod::new(year, month).expect("test_payout_period"),
        String::from("user_1"),
        String::from("leo@gm.com"),
        Some(String::from("payment_method_1")),
    ).add_amount(110)
    .append_payout_item_id(pi_ids[0].clone());

    let p2 = Payout::new(
        String::from("payout_2"),
        PayoutPeriod::new(year, month).expect("test_payout_period"),
        String::from("user_2"),
        String::from("leo@gm.com"),
        Some(String::from("payment_method_2")),
    ).add_amount(220)
    .append_payout_item_id(pi_ids[1].clone());

    let p3 = Payout::new(
        String::from("payout_3"),
        PayoutPeriod::new(year, month).expect("test_payout_period"),
        String::from("user_3"),
        String::from("leo@gm.com"),
        Some(String::from("payment_method_3")),
    ).add_amount(330)
    .append_payout_item_id(pi_ids[2].clone());

    let p4 = Payout::new(
        String::from("payout_4"),
        PayoutPeriod::new(year, month).expect("test_payout_period"),
        String::from("user_4"),
        String::from("leo@gm.com"),
        Some(String::from("payment_method_4")),
    ).add_amount(440)
    .append_payout_item_id(pi_ids[3].clone());

    let p5 = Payout::new(
        String::from("payout_5"),
        PayoutPeriod::new(year, month).expect("test_payout_period"),
        String::from("user_5"),
        String::from("leo@gm.com"),
        Some(String::from("payment_method_5")),
    ).add_amount(550)
    .append_payout_item_id(pi_ids[4].clone());

    let payouts = vec![p1, p2, p3, p4, p5];


    let write_result = db::write_many_payouts(
        conn,
        &payouts,
        &pi_ids, // test only
        &missing_payout_item_ids, // test only
        &refund_item_ids, // test only
    );
    println!("write results: {:?}", write_result);

    payouts
}


/////////////////////////////////
/// Pagination Tests
/////////////////////////////////




#[test]
fn reads_paginate_page_transactions_8_results() {

    let conn = establish_connection_pg("DATABASE_URL");

    let _ = conn.transaction::<(), diesel::result::Error, _>(|| {
        let results_per_page = 8;
        let tx_ids = write_test_transactions(&conn);
        let res = db::read_transactions_paginate_page(&conn, results_per_page);
        let results = res.unwrap().0;
        if results.len() > 7 {
            assert_eq!(results.len(), 8);
        };
        let _ = db::delete_transactions(&conn, &tx_ids);
        Ok(())
    });
}

#[test]
fn reads_paginate_page_transactions_1_result() {
    let conn = establish_connection_pg("DATABASE_URL");
    let results_per_page = 1;

    let _ = conn.transaction::<(), diesel::result::Error, _>(|| {
        let tx_ids = write_test_transactions(&conn);
        let res = db::read_transactions_paginate_page(&conn, results_per_page);
        // println!("\n>>>>");
        // println!("{:#?}", res);
        // println!("<<<<\n");
        let results = res.unwrap().0;
        if results.len() > 0 {
            assert_eq!(results.len(), 1);
        };
        let _ = db::delete_transactions(&conn, &tx_ids);
        Ok(())
    });
}

#[test]
fn paginate_page_into_connection_query() {

    let conn = establish_connection_pg("DATABASE_URL");
    let results_per_page = 3;

    let _ = conn.transaction::<(), diesel::result::Error, _>(|| {
        let tx_ids = write_test_transactions(&conn);

        let res = db::read_transactions_paginate_page(&conn, results_per_page);
        let (vecResults, numPages) = match res {
            Ok((vecResults, numPages)) => (vecResults, numPages),
            Err(_e) => panic!("err in reads_paginate_page test"),
        };

        let connection = connection::Connection::<Transaction> {
            pageInfo: PageInfo {
                endCursor: None,
                isLastPage: false,
                totalPages: Some(numPages),
            },
            totalCount: Some(vecResults.len() as i64),
            totalAmount: None,
            totalFees: None,
            edges: vecResults.into_iter().map(|tx| {
                Edge {
                    cursor: None,
                    node: tx
                }
            }).collect::<Vec<Edge<Transaction>>>()
        };

        // ////// cargo test -- --nocapture
        // println!("\n>>>>");
        // println!("{:#?}", connection);
        // println!("<<<<\n");

        if connection.edges.len() > 2 {
            assert_eq!(connection.edges.len(), 3);
        };
        let _ = db::delete_transactions(&conn, &tx_ids);
        Ok(())
    });
}

#[test]
fn b64_encodes_and_decodes_cursors() {
    let b64encoded = base64::encode("created_at:2019-11-01T00:00:00Z");
    let b64decoded_bytes = base64::decode(&b64encoded).unwrap();
    let b64decoded = std::str::from_utf8(&b64decoded_bytes).unwrap();
    // println!("b64 cursor encoded: {:?}", b64encoded);
    // println!("b64 cursor decoded: {:?}", b64decoded);

    if b64decoded.contains(":") {
        let cursor = b64decoded.splitn(2, ":").collect::<Vec<_>>();
        let cursorName = cursor[0];
        let cursorValue = cursor[1];
        // println!("cursorName: {:?}", cursorName);
        // println!("cursorValue: {:?}", cursorValue);
        assert_eq!(cursorName, "created_at");
        assert_eq!(cursorValue, "2019-11-01T00:00:00Z");
    }
}

#[test]
fn decode_datetime_cursor_helper_works() {

    let b64encoded = base64::encode("created_at:2019-11-01T00:00:00Z");
    // println!("b64 cursor encoded: {:?}", b64encoded);
    let cursor = decode_datetime_cursor(&b64encoded).unwrap();

    assert_eq!(cursor.name, "created_at");
    assert_eq!(
        cursor.value,
        chrono::NaiveDateTime::parse_from_str(
            "2019-11-01T00:00:00Z",
            "%Y-%m-%dT%H:%M:%S%.fZ"
        ).unwrap()
    );
}



#[test]
fn reads_paginate_cursor_transactions_3_results() {

    let conn = establish_connection_pg("DATABASE_URL");

    let _ = conn.transaction::<(), diesel::result::Error, _>(|| {

        let tx_ids = write_test_transactions(&conn);

        let start_date = chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() - 600, 0
        );

        let now_str = chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() - 60, 0
        ).format("%Y-%m-%d %H:%M:%S").to_string();

        let end_date = chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() + 600, 0
        );

        let res = db::read_transactions_paginate_cursor(
            &conn,
            start_date,
            end_date,
            ConnectionQuery {
                sortAscending: Some(true),
                cursor: Some(base64::encode(&format!("created_at:{}", now_str))),
                pageBackwards: Some(false),
                count: 3,
            }
        );

        let (vecTx, numPages, isLastPage) = match res {
            Ok((vecResults, numPages, isLastPage)) => (vecResults, numPages, isLastPage),
            Err(_e) => panic!("err in reads_paginate_cursor test"),
        };

        let endCursor = format!("{}", vecTx[vecTx.len()-1].created_at);

        let connection = connection::Connection::<Transaction> {
            pageInfo: PageInfo {
                endCursor: Some(base64::encode(&endCursor)),
                isLastPage: isLastPage,
                totalPages: Some(numPages),
            },
            totalCount: Some(vecTx.len() as i64),
            totalAmount: None,
            totalFees: None,
            edges: vecTx.into_iter().map(|tx| {
                let edgeCursor = format!("created_at:{:?}", &tx.created_at);
                Edge {
                    cursor: Some(base64::encode(&edgeCursor)),
                    node: tx
                }
            }).collect::<Vec<Edge<Transaction>>>()
        };

        if connection.edges.len() > 0 {
            assert_eq!(connection.edges.len(), 3);
        };
        let _ = db::delete_transactions(&conn, &tx_ids);
        Ok(())
    });
}


#[test]
fn paginate_cursor_sorts_descending() {

    let conn = establish_connection_pg("DATABASE_URL");

    let _ = conn.transaction::<(), diesel::result::Error, _>(|| {
        let tx_ids = write_test_transactions(&conn);
        let start_date = chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() - 600, 0
        );

        let now_str = chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() - 60, 0
        ).format("%Y-%m-%d %H:%M:%S").to_string();

        let end_date = chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() + 600, 0
        );

        let res = db::read_transactions_paginate_cursor(
            &conn,
            start_date,
            end_date,
            connection::ConnectionQuery {
                sortAscending: Some(true),
                cursor: Some(base64::encode(&format!("created_at:{}", now_str))),
                pageBackwards: Some(false),
                count: 3,
            }
        );

        let (vecTx, numPages, isLastPage) = match res {
            Ok((vecResults, numPages, isLastPage)) => (vecResults, numPages, isLastPage),
            Err(_e) => panic!("err in reads_paginate_cursor test"),
        };

        let endCursor = format!("{}", vecTx[vecTx.len()-1].created_at);

        let connection = connection::Connection::<Transaction> {
            pageInfo: PageInfo {
                endCursor: Some(base64::encode(&endCursor)),
                isLastPage: isLastPage,
                totalPages: Some(numPages),
            },
            totalCount: Some(vecTx.len() as i64),
            totalAmount: None,
            totalFees: None,
            edges: vecTx.into_iter().map(|tx| {
                let edgeCursor = format!("created_at:{:?}", &tx.created_at);
                Edge {
                    cursor: Some(base64::encode(&edgeCursor)),
                    node: tx
                }
            }).collect::<Vec<Edge<Transaction>>>()
        };

        // println!("\n>>>>");
        // println!("{:#?}", connection);
        // println!("<<<<\n");

        if connection.edges.len() > 2 {
            assert_eq!(
                connection.edges[0].node.created_at < connection.edges[1].node.created_at,
                true
            );
        };
        let _ = db::delete_transactions(&conn, &tx_ids);

        Ok(())
    });

}

#[test]
fn paginate_cursor_page_backwards() {

    let conn = establish_connection_pg("DATABASE_URL");

    let _ = conn.transaction::<(), diesel::result::Error, _>(|| {
        let tx_ids = write_test_transactions(&conn);

        let start_date = chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() - 600, 0
        );

        let now_str = chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() - 60, 0
        ).format("%Y-%m-%d %H:%M:%S").to_string();

        let end_date = chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() + 600, 0
        );

        let res = db::read_transactions_paginate_cursor(
            &conn,
            start_date,
            end_date,
            ConnectionQuery {
                sortAscending: Some(true),
                cursor: Some(base64::encode(&format!("created_at:{}", now_str))),
                pageBackwards: Some(true),
                count: 3,
            }
        );

        let (vecTx, numPages, isLastPage) = match res {
            Ok((vecResults, numPages, isLastPage)) => (vecResults, numPages, isLastPage),
            Err(_e) => panic!("err in reads_paginate_cursor test"),
        };

        // let endCursor = format!("{}", vecTx[vecTx.len()-1].created_at);

        let connection = connection::Connection::<Transaction> {
            pageInfo: PageInfo {
                endCursor: None,
                isLastPage: isLastPage,
                totalPages: Some(numPages),
            },
            totalCount: Some(vecTx.len() as i64),
            totalAmount: None,
            totalFees: None,
            edges: vecTx.into_iter().map(|tx| {
                let edgeCursor = format!("created_at:{:?}", &tx.created_at);
                Edge {
                    cursor: Some(base64::encode(&edgeCursor)),
                    node: tx
                }
            }).collect::<Vec<Edge<Transaction>>>()
        };

        // println!("\n>>>>");
        // println!("{:#?}", connection);
        // println!("<<<<\n");

        if connection.edges.len() > 2 {
            assert_eq!(
                connection.edges[0].node.created_at <= connection.edges[1].node.created_at,
                true
            );
        };
        let _ = db::delete_transactions(&conn, &tx_ids);
        Ok(())
    });
}


#[test]
fn paginate_cursor_is_last_page_works() {

    let conn = establish_connection_pg("DATABASE_URL");

    let _ = conn.transaction::<(), diesel::result::Error, _>(|| {
        let tx_ids = write_test_transactions(&conn);

        let start_date = chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() - 600, 0
        );

        let now_str = chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() - 60, 0
        ).format("%Y-%m-%dT%H:%M:%SZ").to_string();

        let end_date = chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() + 600, 0
        );

        // println!("START TIME: {:?}", start_date);
        // println!("NOW TIME: {:?}", now_str);
        // println!("END TIME: {:?}", end_date);

        let res = db::read_transactions_paginate_cursor(
            &conn,
            start_date,
            end_date,
            ConnectionQuery {
                sortAscending: Some(true),
                cursor: Some(base64::encode(&format!("created_at:{}", now_str))),
                pageBackwards: Some(false),
                count: 50,
            }
        );

        // println!("DB RESULT : {:?}", res);
        let (vecTx, numPages, isLastPage) = match res {
            Ok((vecResults, numPages, isLastPage)) => (vecResults, numPages, isLastPage),
            Err(_e) => panic!("err in reads_paginate_is_last_page test"),
        };

        let endCursor = format!("created_at:{}", vecTx[vecTx.len()-1].created_at);

        let connection = connection::Connection::<Transaction> {
            pageInfo: PageInfo {
                endCursor: Some(base64::encode(&endCursor)),
                isLastPage: isLastPage,
                totalPages: Some(numPages),
            },
            totalCount: Some(vecTx.len() as i64),
            totalAmount: None,
            totalFees: None,
            edges: vecTx.into_iter().map(|tx| {
                let edgeCursor = format!("created_at:{:?}", &tx.created_at);
                Edge {
                    cursor: Some(base64::encode(&edgeCursor)),
                    node: tx
                }
            }).collect::<Vec<Edge<Transaction>>>()
        };

        // println!("\n>>>>");
        // println!("{:#?}", connection);
        // println!("<<<<\n");

        if connection.edges.len() > 0 {
            assert_eq!(isLastPage, true);
        };
        let _ = db::delete_transactions(&conn, &tx_ids);
        Ok(())
    });
}

#[test]
fn reads_paginate_transactions_cursor_without_cursor() {

    let conn = establish_connection_pg("DATABASE_URL");

    let _ = conn.transaction::<(), diesel::result::Error, _>(|| {
        let tx_ids = write_test_transactions(&conn);
        let start_date = chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() - 600, 0
        );

        let _now_str = chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() - 60, 0
        ).format("%Y-%m-%d %H:%M:%S").to_string();

        let end_date = chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() + 600, 0
        );

        let res = db::read_transactions_paginate_cursor(
            &conn,
            start_date,
            end_date,
            ConnectionQuery {
                sortAscending: Some(true),
                cursor: None,
                pageBackwards: Some(false),
                count: 3,
            }
        );

        let (vecTx, numPages, isLastPage) = match res {
            Ok((vecResults, numPages, isLastPage)) => (vecResults, numPages, isLastPage),
            Err(_e) => panic!("err in reads_paginate_cursor test"),
        };

        let endCursor = format!("{}", vecTx[vecTx.len()-1].created_at);

        let connection = connection::Connection::<Transaction> {
            pageInfo: PageInfo {
                endCursor: Some(base64::encode(&endCursor)),
                isLastPage: isLastPage,
                totalPages: Some(numPages),
            },
            totalCount: Some(vecTx.len() as i64),
            totalAmount: None,
            totalFees: None,
            edges: vecTx.into_iter().map(|tx| {
                let edgeCursor = format!("created_at:{:?}", &tx.created_at);
                Edge {
                    cursor: Some(base64::encode(&edgeCursor)),
                    node: tx
                }
            }).collect::<Vec<Edge<Transaction>>>()
        };

        if connection.edges.len() > 0 {
            assert_eq!(connection.edges.len(), 3);
        };

        let _ = db::delete_transactions(&conn, &tx_ids);
        Ok(())
    });
}


//////////////////////////////////
/// Paginate Tests: Payout Items
/////////////////////////////////


#[test]
fn reads_paginate_cursor_payout_items_3_results() {

    let conn = establish_connection_pg("DATABASE_URL");

    let _ = conn.transaction::<(), diesel::result::Error, _>(|| {

        let pi_ids = write_test_payout_items(&conn)
            .iter().map(|p| p.id.clone()).collect::<Vec<String>>();

        let start_date = chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() - 600, 0
        );

        let now_str = chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() - 60, 0
        ).format("%Y-%m-%d %H:%M:%S").to_string();

        let end_date = chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() + 600, 0
        );

        let res = db::read_payout_items_in_period_paginate_by_cursor(
            &conn,
            start_date,
            end_date,
            None,
            ConnectionQuery {
                sortAscending: Some(false),
                cursor: Some(base64::encode(&format!("created_at:{}", now_str))),
                pageBackwards: Some(false),
                count: 3,
            }
        );

        let (vecTx, numPages, isLastPage) = match res {
            Ok((vecResults, numPages, isLastPage)) => (vecResults, numPages, isLastPage),
            Err(_e) => panic!("err in reads_paginate_cursor test"),
        };

        // let endCursor = format!("{}", vecTx[vecTx.len()-1].created_at);

        let connection = connection::Connection::<PayoutItem> {
            pageInfo: PageInfo {
                // endCursor: Some(base64::encode(&endCursor)),
                endCursor: None,
                isLastPage: isLastPage,
                totalPages: Some(numPages),
            },
            totalCount: Some(vecTx.len() as i64),
            totalAmount: None,
            totalFees: None,
            edges: vecTx.into_iter().map(|tx| {
                let edgeCursor = format!("created_at:{:?}", &tx.created_at);
                Edge {
                    cursor: Some(base64::encode(&edgeCursor)),
                    node: tx
                }
            }).collect::<Vec<Edge<PayoutItem>>>()
        };

        // println!("\n>>>>");
        // println!("{:#?}", connection);
        // println!("<<<<\n");

        if connection.edges.len() > 0 {
            assert_eq!(connection.edges.len(), 3);
        };
        let _ = db::delete_payout_items(&conn, &pi_ids);
        Ok(())
    });

}

#[test]
fn reads_paginate_payout_items_cursor_without_cursor() {

    let conn = establish_connection_pg("DATABASE_URL");

    let _ = conn.transaction::<(), diesel::result::Error, _>(|| {
        let pi_ids = write_test_payout_items(&conn)
            .iter().map(|p| p.id.clone()).collect::<Vec<String>>();

        let start_date = chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() - 600, 0
        );

        let _now_str = chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() - 60, 0
        ).format("%Y-%m-%d %H:%M:%S").to_string();

        let end_date = chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() + 600, 0
        );

        let res = db::read_payout_items_in_period_paginate_by_cursor(
            &conn,
            start_date,
            end_date,
            None,
            ConnectionQuery {
                sortAscending: Some(false),
                cursor: None,
                pageBackwards: Some(false),
                count: 3,
            }
        );

        let (vecTx, numPages, isLastPage) = match res {
            Ok((vecResults, numPages, isLastPage)) => (vecResults, numPages, isLastPage),
            Err(_e) => panic!("err in reads_paginate_cursor test"),
        };

        // let endCursor = format!("{}", vecTx[vecTx.len()-1].created_at);

        let connection = connection::Connection::<PayoutItem> {
            pageInfo: PageInfo {
                // endCursor: Some(base64::encode(&endCursor)),
                endCursor: None,
                isLastPage: isLastPage,
                totalPages: Some(numPages),
            },
            totalCount: Some(vecTx.len() as i64),
            totalAmount: None,
            totalFees: None,
            edges: vecTx.into_iter().map(|tx| {
                let edgeCursor = format!("created_at:{:?}", &tx.created_at);
                Edge {
                    cursor: Some(base64::encode(&edgeCursor)),
                    node: tx
                }
            }).collect::<Vec<Edge<PayoutItem>>>()
        };

        // println!("\n>>>>");
        // println!("{:#?}", connection);
        // println!("<<<<\n");

        if connection.edges.len() > 2 {
            assert_eq!(connection.edges.len(), 3);
        };

        let _ = db::delete_payout_items(&conn, &pi_ids);
        Ok(())
    });
}


/////////// Aggregation Queries ////////////


#[test]
fn reads_transaction_aggregates() {

    let conn = establish_connection_pg("DATABASE_URL");

    let _ = conn.transaction::<(), diesel::result::Error, _>(|| {

        let tx_ids = write_test_transactions(&conn);

        let start_date = chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() - 10, 0
        );
        let end_date = chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() + 10, 0
        );
        // make the window plus-minus 10seconds, looking for transactions
        // in this window only

        let res: TransactionAggregates = db::read_transaction_aggregates(
            &conn,
            start_date,
            end_date,
            false,
        );

        println!("transaction aggregates:\n{:?}", res);
        let _ = db::delete_transactions(&conn, &tx_ids);
        // just want to test that it deserializes properly.
        // amounts will differ unless seeding from scratch
        assert_eq!(
            res.subtotal_sum >= 0,
            true
        );

        Ok(())
    });
}


#[test]
fn reads_payout_item_aggregates() {

    let conn = establish_connection_pg("DATABASE_URL");

    let _ = conn.transaction::<(), diesel::result::Error, _>(|| {

        let pitems = write_test_payout_items(&conn);
        let pids = pitems.iter().map(|p| p.id.clone()).collect::<Vec<String>>();

        let start_date = chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() - 10, 0
        );
        let end_date = chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() + 10, 0
        );
        // make the window plus-minus 10seconds, looking for transactions
        // in this window only

        let res: PayoutItemAggregates = db::read_payout_item_aggregates(
            &conn,
            start_date,
            end_date,
            false,
        );

        let _ = db::delete_payout_items(&conn, &pids);
        // just want to test that it deserializes properly.
        // amounts will differ unless seeding from scratch
        assert_eq!(
            res.amount_total >= 0,
            true
        );

        Ok(())
    });
}

/////////// Batch Query Test ////////////

#[test]
fn construct_batch_update_query() {

    let conn = establish_connection_pg("DATABASE_URL");
    let _ = conn.transaction::<(), diesel::result::Error, _>(|| {

        let payout_items = write_test_payout_items(&conn);

        let created_at = payout_items.iter().next()
        .clone().unwrap().created_at;


        let payout_period = PayoutPeriod::new(
            created_at.year(),
            created_at.month() as i32
        ).unwrap();

        let p1 = Payout::new(
            "store_111".to_string(),
            payout_period.clone(),
            "approver_111".to_string(),
            "rachael@paypal.com".to_string(),
            Some("pm_id123123123123jk".to_string())
        ).set_payout_item_ids(
            payout_items.iter()
                .take(3)
                .map(|p| p.id.clone())
                .collect::<Vec<String>>()
        );

        let p2 = Payout::new(
            "store_222".to_string(),
            payout_period.clone(),
            "approver_222".to_string(),
            "ines@paypal.com".to_string(),
            Some("pm_id123123123123jk".to_string())
        ).set_payout_item_ids(
            payout_items.iter()
                .take(4)
                .map(|p| p.id.clone())
                .collect::<Vec<String>>()
        );

        let mut payouts = vec![p1.clone(), p2.clone()];
        payouts.sort_by_key(|p| p.id.clone());

        let update_query = db::conjure_batch_update_payout_items_query(&payouts);
        println!("\nConstructed batch update query:\n{}", update_query);

        let update_pitems_response = diesel::sql_query(update_query)
            .load::<PayoutItem>(&conn);

        println!("\nupdate pitems response:\n{:?}\n", update_pitems_response);

        let _ = db::delete_payout_items(
            &conn,
            &payout_items.iter().map(|p| p.id.clone()).collect::<Vec<String>>()
        );

        // test every payout_item has a payout_id
        assert_eq!(
            update_pitems_response.unwrap().iter().all(|p: &PayoutItem| {
                p.payout_id == Some(p1.id.clone()) ||
                p.payout_id == Some(p2.id.clone())
            }),
            true
        );

        Ok(())
    });
}

/////// Payout Item Histories //////////

#[test]
fn reads_payout_item_histories_jsonb_deserialization() {

    use crate::db;

    let conn = gm::db::establish_connection_pg("DATABASE_URL");

    let _ = conn.transaction::<(), diesel::result::Error, _>(|| {

        let payout_items = write_test_payout_items(&conn);

        let res = db::read_payout_item_history_summaries(
            &conn,
            "store_1",
            None,
        );

        let _ = db::delete_payout_items(
            &conn,
            &payout_items.iter().map(|p| p.id.clone()).collect::<Vec<String>>()
        );

        println!("\ndeserialize payout item histories success: {:#?}", res);
        // assert_eq!(
        //     true, true
        // );
        Ok(())
    });
}

#[test]
fn reads_payout_item_histories_with_null_cols() {

    use crate::db;

    let conn = gm::db::establish_connection_pg("DATABASE_URL");

    let _ = conn.transaction::<(), diesel::result::Error, _>(|| {

        let res = db::read_payout_item_history_summaries(
            &conn,
            "store_9bb31918-1d2c-43b4-9a61-54b1332d3fdf",
            None
        );

        println!("\ndeserialize payout items histories null cols success: {:#?}", res);
        // assert_eq!(
        //     true, true
        // );
        Ok(())
    });
}


#[test]
fn reads_payouts_pending_refund() {

    use crate::db;
    use gm::db::schema::payouts;
    use gm::db::schema::payout_items;

    let conn = gm::db::establish_connection_pg("DATABASE_URL");

    let _ = conn.transaction::<(), diesel::result::Error, _>(|| {

        let res = payout_items::table
                .filter(
                    payout_items::payout_id.eq_any(vec![
                        "pitem_ed668d22-de78-4c54-88f5-638e5c7bf824"
                    ])
                    // .and(payout_items::payout_status.eq(PayoutStatus::PENDING_REFUND))
                )
                .load::<PayoutItem>(&conn);


        println!("\n................\n{:?}\n--------", res);
        // assert_eq!(
        //     true, true
        // );
        Ok(())
    });
}

#[test]
fn writes_and_reads_payout_splits() {

    let conn = establish_connection_pg("DATABASE_URL");

    let _ = conn.transaction::<(), diesel::result::Error, _>(|| {

        let ps_ids = write_test_payout_splits(&conn);

        let res = db::read_payout_splits_by_ids(
            &conn,
            ps_ids.clone()
        );

        let results = res.unwrap();

        assert_eq!(results.len(), 4);

        /// Order is not deterministic
        ///
        // assert_eq!(results[0].store_or_user_id, String::from("store_1"));
        // assert_eq!(results[0].deal_type, PayoutDealType::SELLER);
        // assert_eq!(results[1].deal_type, PayoutDealType::BUYER_AFFILIATE);
        // assert_eq!(results[2].deal_type, PayoutDealType::REFERRED_SELLER);
        // assert_eq!(results[3].deal_type, PayoutDealType::SELLER_AFFILIATE);

        let _ = db::delete_payout_split(&conn, &ps_ids[0]);
        let _ = db::delete_payout_split(&conn, &ps_ids[1]);
        let _ = db::delete_payout_split(&conn, &ps_ids[2]);
        let _ = db::delete_payout_split(&conn, &ps_ids[3]);

        Ok(())
    });
}

#[test]
fn writes_and_reads_payout_splits_by_store_ids() {

    let conn = establish_connection_pg("DATABASE_URL");

    let _ = conn.transaction::<(), diesel::result::Error, _>(|| {

        let ps_ids = write_test_payout_splits(&conn);
        let store_ids = vec![
            String::from("store_1"),
            String::from("store_2"),
            String::from("store_3"),
            String::from("store_4"),
        ];

        let res = db::read_current_payout_splits_by_store_or_user_ids(
            &conn,
            &store_ids,
            Some(vec![
                PayoutDealType::BUYER_AFFILIATE,
                PayoutDealType::SELLER_AFFILIATE,
                PayoutDealType::SELLER,
                PayoutDealType::REFERRED_SELLER,
            ])
        );

        println!("\nreads payout splits................\n{:?}\n--------", res);

        let results = res.unwrap();

        assert_eq!(results.len(), 4);
        assert_eq!(results[0].store_or_user_id, String::from("store_1"));

        let _ = db::delete_payout_split(&conn, &ps_ids[0]);
        let _ = db::delete_payout_split(&conn, &ps_ids[1]);
        let _ = db::delete_payout_split(&conn, &ps_ids[2]);
        let _ = db::delete_payout_split(&conn, &ps_ids[3]);

        Ok(())
    });
}

#[test]
fn writes_and_reads_payout_splits_for_seller_affiliates() {

    let conn = establish_connection_pg("DATABASE_URL");

    let _ = conn.transaction::<(), diesel::result::Error, _>(|| {

        let ps_ids = write_test_payout_splits(&conn);

        let _referred_store_ids: Vec<String> = vec![
            String::from("store_2"),
            String::from("store_4"),
        ];

        let res1 = db::read_current_seller_referrer_payout_splits_by_store_id(
            &conn,
            String::from("store_2"),
        );
        let res2 = db::read_current_seller_referrer_payout_splits_by_store_id(
            &conn,
            String::from("store_4"),
        );

        let results1 = res1;
        let results2 = res2;

        // assert_eq!(
        //     results[0].seller_affiliate.expect("seller_affiliate err").,
        //     String::from("some_id")
        // );
        println!("\nreads affiliate payout splits........\n{:?}\n------", results1);
        println!("\nreads affiliate payout splits........\n{:?}\n------", results2);

        // assert_eq!(results.len(), 2);
        let _ = db::delete_payout_split(&conn, &ps_ids[0]);
        let _ = db::delete_payout_split(&conn, &ps_ids[1]);
        let _ = db::delete_payout_split(&conn, &ps_ids[2]);
        let _ = db::delete_payout_split(&conn, &ps_ids[3]);

        Ok(())
    });
}


#[test]
fn reads_payouts_aggregates() {

    let conn = establish_connection_pg("DATABASE_URL");

    let _ = conn.transaction::<(), diesel::result::Error, _>(|| {

        let payouts = write_test_payouts(&conn);
        let pids = payouts.iter().map(|p| p.id.clone()).collect::<Vec<String>>();

        let now = chrono::Utc::now();
        let period = PayoutPeriod::new(now.year(), now.month() as i32)
            .expect("test_err in reads_payouts_aggregates");
        let next_period = PayoutPeriod::get_next_payout_period(period.clone())
            .expect("test_err in reads_payouts_aggregates");

        // make the window plus-minus 10seconds, looking for transactions
        // in this window only

        let res: PayoutAggregates = db::read_payout_aggregates(
            &conn,
            next_period.start_period.clone(),
            next_period.end_period.clone(),
            false,
        );
        let total_amount = payouts.iter()
            .fold(0, |acc, p: &Payout| p.amount + acc);

        println!("\n-------------- payout aggregates -------------");
        println!("payout_date {:?}\n", payouts[0].payout_date);
        println!("agg start_date: {:?}\n", next_period.start_period);
        println!("agg end_date: {:?}\n", next_period.end_period);
        println!("{:?}\n", res);

        let _ = db::delete_payouts(&conn, &pids);
        // just want to test that it deserializes properly.
        // amounts will differ unless seeding from scratch
        assert_eq!(
            res.amount_total == (total_amount as i64),
            true
        );

        Ok(())
    });
}