use chrono::Datelike;
use crate::models::MonthError;
use crate::models::ErrJson;

pub const PAYDAY: u32 = 15;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PayoutPeriod {
    pub start_period: chrono::NaiveDateTime,
    pub end_period: chrono::NaiveDateTime,
    pub payout_date: chrono::NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PayoutPeriodType {
    START_PERIOD,
    END_PERIOD,
    PAYOUT_DATE,
}

impl PayoutPeriod {
    pub fn new(year: i32, month: i32) -> Result<PayoutPeriod, MonthError> {
        get_payout_period(year, month as u32)
    }

    pub fn get_next_payout_period(
        current_payout_period: PayoutPeriod
    ) -> Result<PayoutPeriod, MonthError> {
        // payouts for the period 1May ~ 1June are created on 15June,
        // so need to increment payoutPeriod from 1May ~ 1June to 1June ~ 1July
        // to get the payouts created for the May period.
        let (
            next_period_year,
            next_period_month
        ) = increment_period_by_month(
            current_payout_period.start_period.year(),
            current_payout_period.start_period.month(),
        )?;

        get_payout_period(next_period_year, next_period_month)
    }

    fn to_string(&self, payout_period_type: PayoutPeriodType) -> String {
        let (ptype, pday) = match payout_period_type {
            PayoutPeriodType::START_PERIOD => (&self.start_period, 1),
            PayoutPeriodType::END_PERIOD => (&self.end_period, 1),
            PayoutPeriodType::PAYOUT_DATE => (&self.payout_date, PAYDAY),
        };
        match ptype.month() {
            1 => format!("{} Jan {}", pday, ptype.year()),
            2 => format!("{} Feb {}", pday, ptype.year()),
            3 => format!("{} Mar {}", pday, ptype.year()),
            4 => format!("{} Apr {}", pday, ptype.year()),
            5 => format!("{} May {}", pday, ptype.year()),
            6 => format!("{} Jun {}", pday, ptype.year()),
            7 => format!("{} Jul {}", pday, ptype.year()),
            8 => format!("{} Aug {}", pday, ptype.year()),
            9 => format!("{} Sep {}", pday, ptype.year()),
            10 => format!("{} Oct {}", pday, ptype.year()),
            11 => format!("{} Nov {}", pday, ptype.year()),
            12 => format!("{} Dec {}", pday, ptype.year()),
            _ => panic!("impossibru month!"),
        }
    }
}


fn get_payout_period(year: i32, month: u32) -> Result<PayoutPeriod, MonthError> {

    let sec = chrono::NaiveTime::from_hms(0, 0, 0);

    let (
        next_period_year,
        next_period_month
    ) = increment_period_by_month(year, month)?;

    Ok(PayoutPeriod {
        start_period: chrono::NaiveDateTime::new(
            chrono::NaiveDate::from_ymd(
                year,
                month,
                01
            ),
            sec,
        ),
        end_period: chrono::NaiveDateTime::new(
            chrono::NaiveDate::from_ymd(
                next_period_year,
                next_period_month,
                01
            ),
            sec,
        ),
        payout_date: chrono::NaiveDateTime::new(
            chrono::NaiveDate::from_ymd(
                next_period_year,
                next_period_month,
                15
            ),
            sec,
        ),
    })
}


fn increment_period_by_month(year: i32, month: u32) -> Result<(i32, u32), MonthError> {
    match month {
        1..=11 => Ok((year, month + 1)),
        12 => Ok((year + 1, 1)),
        _ => Err(MonthError::ImpossibruMonth(
                errJson!("impossibru month! Must be in 1 to 12.")))
    }
}


pub fn get_one_year_from_now() -> chrono::NaiveDateTime {

    let now = chrono::Utc::now();
    let sec = chrono::NaiveTime::from_hms(0, 0, 0);

    chrono::NaiveDateTime::new(
        chrono::NaiveDate::from_ymd(
            now.year() + 1,
            now.month(),
            now.day()
        ),
        sec
    )
}


#[test]
fn payperiod_start_date_is_as_expected() {
    assert_eq!(
        PayoutPeriod::new(2019, 12).unwrap().start_period,
        chrono::NaiveDateTime::new(
            chrono::NaiveDate::from_ymd(2019, 12, 1),
            chrono::NaiveTime::from_hms(0, 0, 0)
        ),
    );
}

#[test]
fn payperiod_end_date_is_as_expected() {
    assert_eq!(
        PayoutPeriod::new(2019, 3).unwrap().end_period,
        chrono::NaiveDateTime::new(
            chrono::NaiveDate::from_ymd(2019, 4, 1),
            chrono::NaiveTime::from_hms(0, 0, 0)
        ),
    );
}

#[test]
fn payperiod_payout_date_is_as_expected() {
    assert_eq!(
        PayoutPeriod::new(2019, 7).unwrap().payout_date,
        chrono::NaiveDateTime::new(
            chrono::NaiveDate::from_ymd(2019, 8, 15),
            chrono::NaiveTime::from_hms(0, 0, 0)
        ),
    );
}

#[test]
fn next_payperiod_start_date_is_as_expected() {

    let period = PayoutPeriod::new(2019, 12).unwrap();
    // expect next payoutperiod to be 2020, Jan 1st

    assert_eq!(
        PayoutPeriod::get_next_payout_period(period).unwrap().start_period,
        chrono::NaiveDateTime::new(
            chrono::NaiveDate::from_ymd(2020, 1, 1),
            chrono::NaiveTime::from_hms(0, 0, 0)
        ),
    );
}

#[test]
fn next_payperiod_end_date_is_as_expected() {

    let period = PayoutPeriod::new(2019, 3).unwrap();
    // expect next payoutperiod to be 2019-04-01
    // so end date for period is 2019-05-01

    assert_eq!(
        PayoutPeriod::get_next_payout_period(period).unwrap().end_period,
        chrono::NaiveDateTime::new(
            chrono::NaiveDate::from_ymd(2019, 5, 1),
            chrono::NaiveTime::from_hms(0, 0, 0)
        ),
    );
}

#[test]
fn next_payperiod_payout_date_is_as_expected() {

    let period = PayoutPeriod::new(2019, 7).unwrap();
    // expect next payoutperiod to be 2019-08-01
    // so next payout date is 2019-09-15

    assert_eq!(
        PayoutPeriod::get_next_payout_period(period).unwrap().payout_date,
        chrono::NaiveDateTime::new(
            chrono::NaiveDate::from_ymd(2019, 9, 15),
            chrono::NaiveTime::from_hms(0, 0, 0)
        ),
    );
}