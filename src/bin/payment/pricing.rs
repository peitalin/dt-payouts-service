use crate::models::{
    PayoutSplit,
    PayoutItem,
    PayoutDealType,
};

/// Fallback Variables
/// Used if not defined in docker-compose

pub static PAYMENT_FEE_PERCENTAGE: f64 = 0.036;
// apply 3.6% payment processing fee

pub static PAYMENT_FEE_FIXED: i32 = 30;
// 30c Stripe fee

pub static PLATFORM_FEE_PERCENTAGE: f64 = 0.15;
pub static SELLER_FEE_PERCENTAGE: f64 = 1.0 - PLATFORM_FEE_PERCENTAGE;
// apply 15% platform fee

pub static SELLER_AFFILIATE_FEE_PERCENTAGE: f64 = 0.05;
// seller affiliate's default cut is 5%

pub static BUYER_AFFILIATE_FEE_PERCENTAGE: f64 = 0.25;
// WARNING: changing this, and you should also change the variable in user-service
// buyer affiliate's default cut is 25%
pub static MAX_BUYER_AFFILIATE_FEE_PERCENTAGE: f64 = 0.5;
// (the largest custom buyer affiliate deal we can offer)


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaymentFees {
    pub payment_fee_percentage: f64,
    pub payment_fee_fixed: i32,
    pub platform_fee_percentage: f64,
}

impl PaymentFees {
    pub fn new() -> Self {
        Self {
            payment_fee_percentage: PAYMENT_FEE_PERCENTAGE,
            payment_fee_fixed: PAYMENT_FEE_FIXED,
            platform_fee_percentage: PLATFORM_FEE_PERCENTAGE
        }
    }

    pub fn calculate_payment_processing_fee(
        &self,
        subtotal: i32,
    ) -> i32 {
        let fee_per_item = ((subtotal as f64) * self.payment_fee_percentage).round() as i32 + self.payment_fee_fixed;
        fee_per_item
    }
}




pub fn calculate_platform_fees(
    subtotal: i32,
    payment_proc_fee: i32, // incomiing payment_process_fee from shopping
    seller_payout_split: Option<PayoutSplit>,
    buyer_aff_payout_split: Option<PayoutSplit>,
    seller_aff_payout_split: Option<PayoutSplit>,
    created_at: Option<chrono::NaiveDateTime>,
    // set by frenzy as mock_date or
    // read from Stripe.PaymentIntent.createdAt
    // needs to be set for testing
) -> CalculatedEarnings {

    let payment_processing_fee = match payment_proc_fee {
        0 => PaymentFees::new().calculate_payment_processing_fee(subtotal),
        _ => payment_proc_fee,
    };
    // if payment_processing_fee from shopping service is 0, (buyer paid 0)
    // then calculate the fee for seller to pay
    // 3.6% of subtotal, plus 30c per transaction

    let seller_rate = check_rate_expiry_for_seller(seller_payout_split, created_at);
    let buyer_aff_rate = check_rate_expiry_for_affiliate(buyer_aff_payout_split, created_at);
    let seller_aff_rate = check_rate_expiry_for_affiliate(seller_aff_payout_split, created_at);
    debug!("Seller rate: {}", seller_rate);
    debug!("Seller Affiliate rate: {}", seller_aff_rate);
    debug!("Buyer Affiliate rate: {}", buyer_aff_rate);

    generate_earnings_from_payout_splits(
        GenerateEarningsInput {
            subtotal: subtotal,
            payment_processing_fee: payment_processing_fee,
            seller_rate: seller_rate,
            buyer_aff_rate: buyer_aff_rate,
            seller_aff_rate: seller_aff_rate,
        }
    )
}

fn check_rate_expiry_for_seller(
    payout_split: Option<PayoutSplit>,
    created_at: Option<chrono::NaiveDateTime>,
) -> f64 {

    let now: chrono::NaiveDateTime = match created_at {
        Some(date) => date,
        None => chrono::NaiveDateTime::from_timestamp(
                    chrono::Utc::now().timestamp(), 0)
    };

    match payout_split {
        // no payout_split, revert to default platform fee for seller
        None => 1.0 - PLATFORM_FEE_PERCENTAGE,
        Some(ps) => match ps.expires_at {
            None => ps.rate, // payout_split with no expiry, use it
            Some(exp) => {
                if exp > now {
                    debug!("PayoutSplit expires at: {:?}, still valid", exp);
                    // payout_split valid, use its rate
                    ps.rate
                } else {
                    // payout_split expired, revert to default platform fee
                    1.0 - PLATFORM_FEE_PERCENTAGE
                }
            }
        }
    }
}


fn check_rate_expiry_for_affiliate(
    payout_split: Option<PayoutSplit>,
    created_at: Option<chrono::NaiveDateTime>,
) -> f64 {

    let now: chrono::NaiveDateTime = match created_at {
        Some(date) => date,
        None => chrono::NaiveDateTime::from_timestamp(
                    chrono::Utc::now().timestamp(), 0)
    };

    match payout_split {
        // no payout_split, affiliate gets nothing by default
        None => 0.0,
        Some(ps) => match ps.expires_at {
            None => ps.rate, // payout_split with no expiry, use it
            Some(exp) => {
                if exp > now {
                    debug!("PayoutSplit expires at: {:?}, still valid", exp);
                    // payout_split valid, use its rate
                    ps.rate
                } else {
                    // payout_split expired, revert to default affiliate fee of 0
                    0.0
                }
            }
        }
    }
}

pub struct GenerateEarningsInput {
    pub subtotal: i32,
    pub payment_processing_fee: i32,
    pub seller_rate: f64,
    pub buyer_aff_rate: f64,
    pub seller_aff_rate: f64,
}

pub struct CalculatedEarnings {
    pub seller_earnings_less_payment_fee: i32,
    pub payment_processing_fee: i32,
    pub gm_earnings: i32,
    pub buyer_affiliate_earnings: i32,
    pub seller_affiliate_earnings: i32,
}


pub fn generate_earnings_from_payout_splits(
    generate_earnings_input: GenerateEarningsInput
) -> CalculatedEarnings {

    //// How the split works
    ///
    ///                total
    ///              /       \
    ///    seller-earnings    platform-earnings
    ///      /        \             /         \
    ///   baff       seller       saff        gm
    ///
    ///
    /// saff: Seller affiliate
    /// baff: Buyer affiliate
    /// Seller also pays payment processing fee, so enough will always remain to cover that.

    // Parameters:
    let g = generate_earnings_input;
    let subtotal         = g.subtotal;
    let payment_proc_fee = g.payment_processing_fee;
    let seller_rate      = g.seller_rate;
    let buyer_aff_rate   = g.buyer_aff_rate;
    let seller_aff_rate  = g.seller_aff_rate;

    // 1. split total payments between relay and the seller
    let seller_earnings = (subtotal.clone() as f64 * seller_rate).ceil();
    let relay_share = subtotal.clone() as f64 - seller_earnings;

    // 2. split relay's share into 2 for seller affiliates
    // NOTE: This is just to support legacy program.
    // (usually 100% of this share stays with relay)
    let (
      gm_earnings,
      seller_aff_earnings,
    ) = match seller_aff_rate {
        x if x <= PLATFORM_FEE_PERCENTAGE => {

            // 0.05 / 0.15 = 1/3. Seller affiliate get 5% of the subtotal,
            // which is 33% of gm-platform's 15% earnings
            // Calculate affiliate earnings as a percentage (33%) of relay's share
            let seller_aff_earnings = (
                seller_aff_rate * relay_share.clone() / PLATFORM_FEE_PERCENTAGE
            ).round() as i32;

            // calculate gm-platform-fee as the remainder to prevent rounding cents
            let gm_earnings: i32 = (relay_share.clone() as i32) - seller_aff_earnings;
            (gm_earnings, seller_aff_earnings)
        },
        // assert that affiliate take rate never exceeds 100% of our share
        _ => {
            // if fee exceeds our share, set gm fee as 0.
            let gm_earnings = 0 as i32;
            let seller_aff_earnings = relay_share.clone() as i32;
            (gm_earnings, seller_aff_earnings)
        }
    };

    // 3. split seller's share into 2 for buyer affiliates...

    // Calculate the max rate a buyer affiliate can get, given that we may have custom
    // deals. The trick is this side has to pay the payment processing fee, so the affiliate
    // can't take so much that there's nothing left for the seller to cover that. Hence the
    // use of a pre-determined upper limit.
    let applied_buyer_aff_rate = buyer_aff_rate.min(MAX_BUYER_AFFILIATE_FEE_PERCENTAGE);
    let buyer_aff_earnings = (
        applied_buyer_aff_rate * seller_earnings.clone() / SELLER_FEE_PERCENTAGE
    ).round() as i32;
    let seller_earnings_after_affiliate: i32 = (seller_earnings.clone() as i32) - buyer_aff_earnings;

    // seller_earnings_after_affiliate - payment_proc_fee is always > 0
    CalculatedEarnings {
        seller_earnings_less_payment_fee: (seller_earnings_after_affiliate as i32) - payment_proc_fee,
        payment_processing_fee: payment_proc_fee,
        gm_earnings: gm_earnings,
        buyer_affiliate_earnings: buyer_aff_earnings,
        seller_affiliate_earnings: seller_aff_earnings,
    }

    // OLD WAY BELOW FYI (when affiliate takings came out of platform fees)

    // //// How the split works
    // ///
    // ///                total
    // ///              /       \
    // ///           relay      seller
    // ///        /    |    \
    // ///    saff   baff  gm
    // ///
    // /// saff: Seller affiliate
    // /// baff: Buyer affiliate

    // // Parameters:
    // let g = generate_earnings_input;
    // let subtotal         = g.subtotal;
    // let payment_proc_fee = g.payment_processing_fee;
    // let seller_rate      = g.seller_rate;
    // let buyer_aff_rate   = g.buyer_aff_rate;
    // let seller_aff_rate  = g.seller_aff_rate;

    // // 1. split total payments between relay and the seller
    // let seller_earnings = (subtotal.clone() as f64 * seller_rate).ceil();
    // let relay_share = subtotal.clone() as f64 - seller_earnings;

    // // 2. split relay's share into 3 for affiliates
    // let (
    //     gm_earnings,
    //     buyer_aff_earnings,
    //     seller_aff_earnings,
    // ) = match buyer_aff_rate + seller_aff_rate {
    //     x if x <= PLATFORM_FEE_PERCENTAGE => {

    //         // 0.05 / 0.15 = 1/3. Buyer affiliate get 5% of the subtotal,
    //         // which is 33% of gm-platform's 15% earnings
    //         // Calculate affiliate earnings as a percentage (33%) of relay's share
    //         let buyer_aff_earnings = (
    //             relay_share.clone() * buyer_aff_rate / PLATFORM_FEE_PERCENTAGE
    //         ).round() as i32;

    //         let seller_aff_earnings = (
    //             relay_share.clone() * seller_aff_rate / PLATFORM_FEE_PERCENTAGE
    //         ).round() as i32;

    //         // calculate gm-platform-fee as the remainder to prevent rounding cents
    //         let gm_earnings: i32 = (relay_share.clone() as i32) - buyer_aff_earnings - seller_aff_earnings;
    //         (gm_earnings, buyer_aff_earnings, seller_aff_earnings)
    //     },
    //     // assert that affiliate take rate never exceeds 100% of our share
    //     _ => {
    //         // if fee exceeds our share, set gm fee as 0.
    //         let gm_earnings = 0 as i32;
    //         // then split the fee between seller-affiliate and buyer-affiliate 50/50
    //         let buyer_aff_earnings = (relay_share.clone() * 0.5 * PLATFORM_FEE_PERCENTAGE).ceil() as i32;
    //         // calculate as remainder to prevent rounding cents errors
    //         let seller_aff_earnings = relay_share.clone() as i32 - buyer_aff_earnings;
    //         (gm_earnings, buyer_aff_earnings, seller_aff_earnings)
    //     }
    // };

    // CalculatedEarnings {
    //     seller_earnings_less_payment_fee: (seller_earnings as i32) - payment_proc_fee,
    //     payment_processing_fee: payment_proc_fee,
    //     gm_earnings: gm_earnings,
    //     buyer_affiliate_earnings: buyer_aff_earnings,
    //     seller_affiliate_earnings: seller_aff_earnings,
    // }
}

// /// Takes a function, returns a function
// pub fn apply_default_platform_fee<F>(f: F) -> impl Fn(i32, i32) -> (i32, i32, i32)
//     where F: Fn(f64) -> Box<dyn Fn(i32 ,i32) -> (i32, i32, i32)>
// {
//     // apply 15% platform fee
//     f(PLATFORM_FEE_PERCENTAGE)
// }
// // Can't write impl Fn(i32) in trait definitions,
// // So must use Box<dyn Fn()>

// pub fn generate_platform_fees2(
//     platform_fee_percentage: f64,
// ) -> Box<dyn Fn(i32, i32) -> (i32, i32, i32)> {
//     // Later:
//     // 1. Lookup PayoutSplit if response contains PayoutSplitId
//     // calculate splits based on that PayoutSplit from DB
//     Box::new(move |subtotal, payment_processing_fee| {
//         let seller_payment = (subtotal.clone() as f64 * (1.0 - platform_fee_percentage)).ceil() as i32;
//         let platform_fee = (subtotal.clone() as f64 * platform_fee_percentage) as i32;
//         let remainder = subtotal - seller_payment - platform_fee;
//         let affiliate_fee = remainder;
//         (
//             seller_payment - payment_processing_fee,
//             platform_fee,
//             affiliate_fee
//         )
//     })
// }



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_platform_fees_correctly() {
        // 15% platform fees
        // let seller_cut = 850 - (1000.0 * 0.036 + 30.0); // 784
        let CalculatedEarnings {
            seller_earnings_less_payment_fee,
            payment_processing_fee: _,
            gm_earnings,
            seller_affiliate_earnings,
            buyer_affiliate_earnings
        } = calculate_platform_fees(1000, 0, None, None, None, None);

        assert_eq!(seller_earnings_less_payment_fee, 784);
        assert_eq!(gm_earnings, 150);
        assert_eq!(seller_affiliate_earnings, 0);
        assert_eq!(buyer_affiliate_earnings, 0);
    }

    #[test]
    fn calculate_platform_fees_correctly2() {
        // 15% platform fees
        // let seller_cut = 1700 - (2000.0 * 0.036 + 30.0); // 1598
        let CalculatedEarnings {
            seller_earnings_less_payment_fee,
            payment_processing_fee: _,
            gm_earnings,
            seller_affiliate_earnings,
            buyer_affiliate_earnings
        } = calculate_platform_fees(2000, 0, None, None, None, None);

        assert_eq!(seller_earnings_less_payment_fee, 1598);
        assert_eq!(gm_earnings, 300);
        assert_eq!(seller_affiliate_earnings, 0);
        assert_eq!(buyer_affiliate_earnings, 0);
    }

    #[test]
    fn calculate_platform_fees_correctly3() {
        // 15% platform fees
        let subtotal = 1355;
        // let seller_cut = 1151.75 - (1355.0 * 0.036 + 30.0); // 1072.97
        // let fee = 1355.0 * 0.036 + 30.0 // 78.78
        let CalculatedEarnings {
            seller_earnings_less_payment_fee,
            payment_processing_fee: _,
            gm_earnings,
            seller_affiliate_earnings,
            buyer_affiliate_earnings
        } = calculate_platform_fees(subtotal, 0, None, None, None, None);

        assert_eq!(seller_earnings_less_payment_fee, 1073);
        assert_eq!(gm_earnings, 203);
        assert_eq!(seller_affiliate_earnings, 0);
        assert_eq!(buyer_affiliate_earnings, 0);
        assert_eq!(subtotal, 1073 + 203 + 79)
    }

    #[test]
    fn calc_splits_baff_normal_rate() {
        // buyer affiliate normal rate
        // 15% platform fees
        let subtotal = 2345; // 1994 to 351 (seller v platform portions)
        let expectedFee = 114; // (rounded from 114.42)
        let buyer_aff = Some(PayoutSplit::new(
          String::from("store_test1"),
          PayoutDealType::BUYER_AFFILIATE,
          None,
          0.25,
          None,
        ));
        let CalculatedEarnings {
            seller_earnings_less_payment_fee,
            payment_processing_fee: _,
            gm_earnings,
            seller_affiliate_earnings,
            buyer_affiliate_earnings
        } = calculate_platform_fees(subtotal, 0, None, buyer_aff, None, None);

        assert_eq!(gm_earnings, 351);
        assert_eq!(seller_affiliate_earnings, 0);
        assert_eq!(buyer_affiliate_earnings, 586);
        assert_eq!(seller_earnings_less_payment_fee, 1294);
        assert_eq!(subtotal, expectedFee + buyer_affiliate_earnings + seller_affiliate_earnings + gm_earnings + seller_earnings_less_payment_fee);
    }

    #[test]
    fn calc_splits_baff_saff_normal_rate() {
        // buyer affiliate normal rate
        // 15% platform fees
        let subtotal = 2345; // 1994 to 351 (seller v platform portions)
        let expectedFee = 114; // (rounded from 114.42)
        let buyer_aff = Some(PayoutSplit::new(
          String::from("store_test1"),
          PayoutDealType::BUYER_AFFILIATE,
          None,
          0.25,
          None,
        ));
        let seller_aff = Some(PayoutSplit::new(
            String::from("store_test1"),
            PayoutDealType::SELLER_AFFILIATE,
            None,
            0.05,
            None,
          ));
        let CalculatedEarnings {
            seller_earnings_less_payment_fee,
            payment_processing_fee: _,
            gm_earnings,
            seller_affiliate_earnings,
            buyer_affiliate_earnings
        } = calculate_platform_fees(subtotal, 0, None, buyer_aff, seller_aff, None);

        assert_eq!(gm_earnings, 234);
        assert_eq!(seller_affiliate_earnings, 117);
        assert_eq!(buyer_affiliate_earnings, 586);
        assert_eq!(seller_earnings_less_payment_fee, 1294);
        assert_eq!(subtotal, expectedFee + buyer_affiliate_earnings + seller_affiliate_earnings + gm_earnings + seller_earnings_less_payment_fee);
    }

    #[test]
    fn calc_splits_baff_low_rate() {
        // buyer affiliate low rate
        // 15% platform fees
        let subtotal = 2345; // 1994 to 351 (seller v platform portions)
        let expectedFee = 114; // (rounded from 114.42)
        let buyer_aff = Some(PayoutSplit::new(
          String::from("store_test1"),
          PayoutDealType::BUYER_AFFILIATE,
          None,
          0.2,
          None,
        ));
        let CalculatedEarnings {
            seller_earnings_less_payment_fee,
            payment_processing_fee: _,
            gm_earnings,
            seller_affiliate_earnings,
            buyer_affiliate_earnings
        } = calculate_platform_fees(subtotal, 0, None, buyer_aff, None, None);

        assert_eq!(gm_earnings, 351);
        assert_eq!(seller_affiliate_earnings, 0);
        assert_eq!(buyer_affiliate_earnings, 469);
        assert_eq!(seller_earnings_less_payment_fee, 1411);
        assert_eq!(subtotal, expectedFee + buyer_affiliate_earnings + seller_affiliate_earnings + gm_earnings + seller_earnings_less_payment_fee);
    }

    #[test]
    fn calc_splits_baff_higher_rate() {
        // buyer affiliate low rate
        // 15% platform fees
        let subtotal = 2345; // 1994 to 351 (seller v platform portions)
        let expectedFee = 114; // (rounded from 114.42)
        let buyer_aff = Some(PayoutSplit::new(
          String::from("store_test1"),
          PayoutDealType::BUYER_AFFILIATE,
          None,
          0.3,
          None,
        ));
        let CalculatedEarnings {
            seller_earnings_less_payment_fee,
            payment_processing_fee: _,
            gm_earnings,
            seller_affiliate_earnings,
            buyer_affiliate_earnings
        } = calculate_platform_fees(subtotal, 0, None, buyer_aff, None, None);

        assert_eq!(gm_earnings, 351);
        assert_eq!(seller_affiliate_earnings, 0);
        assert_eq!(buyer_affiliate_earnings, 704);
        assert_eq!(seller_earnings_less_payment_fee, 1176);
        assert_eq!(subtotal, expectedFee + buyer_affiliate_earnings + seller_affiliate_earnings + gm_earnings + seller_earnings_less_payment_fee);
    }

    #[test]
    fn calc_splits_baff_above_max_rate() {
        // buyer affiliate low rate
        // 15% platform fees
        let subtotal = 2345; // 1994 to 351 (seller v platform portions)
        let expectedFee = 114; // (rounded from 114.42)
        let buyer_aff = Some(PayoutSplit::new(
          String::from("store_test1"),
          PayoutDealType::BUYER_AFFILIATE,
          None,
          0.6, // clip to 0.5
          None,
        ));
        let CalculatedEarnings {
            seller_earnings_less_payment_fee,
            payment_processing_fee: _,
            gm_earnings,
            seller_affiliate_earnings,
            buyer_affiliate_earnings
        } = calculate_platform_fees(subtotal, 0, None, buyer_aff, None, None);

        assert_eq!(gm_earnings, 351);
        assert_eq!(seller_affiliate_earnings, 0);
        assert_eq!(buyer_affiliate_earnings, 1173);
        assert_eq!(seller_earnings_less_payment_fee, 707);
        assert_eq!(subtotal, expectedFee + buyer_affiliate_earnings + seller_affiliate_earnings + gm_earnings + seller_earnings_less_payment_fee);
    }

    #[test]
    fn calc_splits_saff_above_max_rate() {
        // buyer affiliate low rate
        // 15% platform fees
        let subtotal = 2345; // 1994 to 351 (seller v platform portions)
        let expectedFee = 114; // (rounded from 114.42)
        let seller_aff = Some(PayoutSplit::new(
          String::from("store_test1"),
          PayoutDealType::SELLER_AFFILIATE,
          None,
          0.3,
          None,
        ));
        let CalculatedEarnings {
            seller_earnings_less_payment_fee,
            payment_processing_fee: _,
            gm_earnings,
            seller_affiliate_earnings,
            buyer_affiliate_earnings
        } = calculate_platform_fees(subtotal, 0, None, None, seller_aff, None);

        assert_eq!(gm_earnings, 0);
        assert_eq!(seller_affiliate_earnings, 351);
        assert_eq!(buyer_affiliate_earnings, 0);
        assert_eq!(seller_earnings_less_payment_fee, 1880);
        assert_eq!(subtotal, expectedFee + buyer_affiliate_earnings + seller_affiliate_earnings + gm_earnings + seller_earnings_less_payment_fee);
    }

    #[test]
    fn calc_splits_min_order_no_baff() {
        // buyer affiliate low rate
        // 15% platform fees
        let subtotal = 100; // 85 to 15 (seller v platform portions)
        let expectedFee = 34; // (rounded from 33.6)
        let CalculatedEarnings {
            seller_earnings_less_payment_fee,
            payment_processing_fee: _,
            gm_earnings,
            seller_affiliate_earnings,
            buyer_affiliate_earnings
        } = calculate_platform_fees(subtotal, 0, None, None, None, None);

        assert_eq!(gm_earnings, 15);
        assert_eq!(seller_affiliate_earnings, 0);
        assert_eq!(buyer_affiliate_earnings, 0);
        assert_eq!(seller_earnings_less_payment_fee, 51);
        assert_eq!(subtotal, expectedFee + buyer_affiliate_earnings + seller_affiliate_earnings + gm_earnings + seller_earnings_less_payment_fee);
    }

    #[test]
    fn calc_splits_min_order_with_baff() {
        // buyer affiliate low rate
        // 15% platform fees
        let subtotal = 100; // 85 to 15 (seller v platform portions)
        let expectedFee = 34; // (rounded from 33.6)
        let buyer_aff = Some(PayoutSplit::new(
          String::from("store_test1"),
          PayoutDealType::BUYER_AFFILIATE,
          None,
          0.25,
          None,
        ));
        let CalculatedEarnings {
            seller_earnings_less_payment_fee,
            payment_processing_fee: _,
            gm_earnings,
            seller_affiliate_earnings,
            buyer_affiliate_earnings
        } = calculate_platform_fees(subtotal, 0, None, buyer_aff, None, None);

        assert_eq!(gm_earnings, 15);
        assert_eq!(seller_affiliate_earnings, 0);
        assert_eq!(buyer_affiliate_earnings, 25);
        assert_eq!(seller_earnings_less_payment_fee, 26);
        assert_eq!(subtotal, expectedFee + buyer_affiliate_earnings + seller_affiliate_earnings + gm_earnings + seller_earnings_less_payment_fee);
    }

    #[test]
    fn calc_splits_min_order_with_max_baff() {
        // buyer affiliate low rate
        // 15% platform fees
        let subtotal = 100; // 85 to 15 (seller v platform portions)
        let expectedFee = 34; // (rounded from 33.6)
        let buyer_aff = Some(PayoutSplit::new(
          String::from("store_test1"),
          PayoutDealType::BUYER_AFFILIATE,
          None,
          0.6, // above max, should clip to 0.5
          None,
        ));
        let CalculatedEarnings {
            seller_earnings_less_payment_fee,
            payment_processing_fee: _,
            gm_earnings,
            seller_affiliate_earnings,
            buyer_affiliate_earnings
        } = calculate_platform_fees(subtotal, 0, None, buyer_aff, None, None);

        assert_eq!(gm_earnings, 15);
        assert_eq!(seller_affiliate_earnings, 0);
        assert_eq!(buyer_affiliate_earnings, 50);
        assert_eq!(seller_earnings_less_payment_fee, 1);
        assert_eq!(subtotal, expectedFee + buyer_affiliate_earnings + seller_affiliate_earnings + gm_earnings + seller_earnings_less_payment_fee);
    }

    #[test]
    fn payout_split_affiliate_expired() {

        let expiry = chrono::NaiveDateTime::new(
            chrono::NaiveDate::from_ymd(2015, 1, 12),
            chrono::NaiveTime::from_hms(0, 0, 0),
        );
        let now: chrono::NaiveDateTime = chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp(),
            0
        );

        println!("expiry: {:?}", expiry);
        println!("now: {:?}", now);
        println!(
            "{:?} > {:?}: {:?}",
            expiry,
            now,
            expiry > now
        );

        let expect_rate = 0.0; // expired so rate should be 0 for affiliate
        let input_rate = 0.05;

        let result_rate = check_rate_expiry_for_affiliate(
            Some(PayoutSplit::new(
                String::from("store_test1"),
                PayoutDealType::BUYER_AFFILIATE,
                Some(expiry),
                input_rate,
                None,
            )),
            None,
        );

        println!("\nExpect PayoutSplit.rate to be 0 (expired)");
        println!("expect_rate: {:?}", expect_rate);
        println!("result_rate: {:?}", result_rate);

        assert_eq!(expect_rate, result_rate)
    }

    #[test]
    fn payout_split_affiliate_not_expired() {

        let now = chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp(),
            0,
        );
        let expiry: chrono::NaiveDateTime = chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp() + 1_000_000,
            0
        );

        println!("now: {:?}", now);
        println!("expiry: {:?}", expiry);
        println!(
            "{:?} < {:?}: {:?}",
            now,
            expiry,
            now < expiry
        );

        let expect_rate = 0.05; // expired so rate should be 0 for affiliate
        let input_rate = 0.05;

        let result_rate = check_rate_expiry_for_affiliate(
            Some(PayoutSplit::new(
                String::from("store_test1"),
                PayoutDealType::BUYER_AFFILIATE,
                Some(expiry),
                input_rate,
                None,
            )),
            None,
        );

        println!("\nExpect PayoutSplit.rate to NOT be 0 (not expired)");
        println!("expect_rate: {:?}", expect_rate);
        println!("result_rate: {:?}", result_rate);

        assert_eq!(expect_rate, result_rate)
    }
}

