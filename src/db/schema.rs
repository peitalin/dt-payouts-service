table! {
    payment_method_addresses (payment_method_id) {
        payment_method_id -> Text,
        line1 -> Nullable<Text>,
        line2 -> Nullable<Text>,
        city -> Nullable<Text>,
        state -> Nullable<Text>,
        postal_code -> Nullable<Text>,
        country -> Nullable<Text>,
        town -> Nullable<Text>,
    }
}

table! {
    payment_methods (id) {
        id -> Text,
        user_id -> Text,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
        customer_id -> Nullable<Text>,
        payment_processor -> Nullable<Text>,
        payment_method_types -> Nullable<Array<Text>>,
        last4 -> Nullable<Text>,
        exp_month -> Nullable<Int4>,
        exp_year -> Nullable<Int4>,
        name -> Nullable<Text>,
        email -> Nullable<Text>,
        details -> Nullable<Text>,
    }
}

table! {
    payout_items (id) {
        id -> Text,
        payee_id -> Text,
        payee_type -> Text,
        amount -> Int4,
        payment_processing_fee -> Int4,
        created_at -> Timestamp,
        payout_status -> Text,
        currency -> Text,
        order_item_id -> Text,
        txn_id -> Text,
        payout_id -> Nullable<Text>,
    }
}

table! {
    payout_methods (id) {
        id -> Text,
        payee_id -> Text,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
        payout_type -> Nullable<Text>,
        payout_email -> Nullable<Text>,
        payout_processor -> Nullable<Text>,
        payout_processor_id -> Nullable<Text>,
    }
}

table! {
    payout_splits (id) {
        id -> Text,
        created_at -> Timestamp,
        store_or_user_id -> Text,
        deal_type -> Text,
        expires_at -> Nullable<Timestamp>,
        rate -> Float8,
        referrer_id -> Nullable<Text>,
    }
}

table! {
    payouts (id) {
        id -> Text,
        payee_id -> Text,
        payee_type -> Text,
        amount -> Int4,
        created_at -> Nullable<Timestamp>,
        start_period -> Nullable<Timestamp>,
        end_period -> Nullable<Timestamp>,
        payout_date -> Nullable<Timestamp>,
        payout_status -> Text,
        payout_email -> Text,
        currency -> Text,
        payout_item_ids -> Array<Text>,
        approved_by_ids -> Array<Text>,
        payout_batch_id -> Nullable<Text>,
        details -> Nullable<Text>,
        paid_to_payment_method_id -> Nullable<Text>,
    }
}

table! {
    refunds (id) {
        id -> Text,
        transaction_id -> Text,
        order_id -> Text,
        order_item_ids -> Nullable<Array<Text>>,
        created_at -> Timestamp,
        reason -> Nullable<Text>,
        reason_details -> Nullable<Text>,
    }
}

table! {
    transactions (id) {
        id -> Text,
        subtotal -> Int4,
        taxes -> Int4,
        payment_processing_fee -> Int4,
        created_at -> Timestamp,
        currency -> Nullable<Text>,
        charge_id -> Nullable<Text>,
        customer_id -> Nullable<Text>,
        order_id -> Nullable<Text>,
        payment_processor -> Nullable<Text>,
        payment_method_id -> Nullable<Text>,
        payment_intent_id -> Nullable<Text>,
        refund_id -> Nullable<Text>,
        details -> Nullable<Text>,
    }
}

allow_tables_to_appear_in_same_query!(
    payment_method_addresses,
    payment_methods,
    payout_items,
    payout_methods,
    payout_splits,
    payouts,
    refunds,
    transactions,
);
