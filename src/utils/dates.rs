//// This file contains date functions and deserializer
/// implmentations for dates.
use serde::de::{Deserialize, Deserializer};
use serde::{de, ser};
use std::fmt::Display;
use std::fmt;
use std::ops::{AddAssign, Div};
use std::convert::From;
use num::cast::{FromPrimitive, ToPrimitive};
use failure::Error;


pub fn pick_datetime_format(s: &str) -> &str {
    // Some dates contain T or space
    // 2019-05-18T11:25:01
    // 2019-05-18 11:25:01
    // Some contain milliseconds.
    // Some contain Z timezone (UTC)
    match s.contains("T") {
        true => match s.contains("Z") {
            true => if s.contains(".") {
                "%Y-%m-%dT%H:%M:%S%.fZ"
            } else {
                "%Y-%m-%dT%H:%M:%SZ"
            },
            false => if s.contains(".") {
                "%Y-%m-%dT%H:%M:%S%.f"
            } else {
                "%Y-%m-%dT%H:%M:%S"
            },
        },
        false => match s.contains("Z") {
            true => if s.contains(".") {
                "%Y-%m-%d %H:%M:%S%.fZ"
            } else {
                "%Y-%m-%d %H:%M:%SZ"
            },
            false => if s.contains(".") {
                "%Y-%m-%d %H:%M:%S%.f"
            } else {
                "%Y-%m-%d %H:%M:%S"
            },
        }
    }
}


// "%Y-%m-%dT%H:%M:%SZ" => Option<chrono::NaiveDateTime>
pub fn from_datetimestr_to_option_naivedatetime<'de, D>(
    deserializer: D,
) -> Result<Option<chrono::NaiveDateTime>, D::Error>
where
    D: de::Deserializer<'de>,
{
    // define a visitor that deserializes Option to NaiveDateTime
    struct OptionNaiveDateTimeVisitor;
    // // define a visitor that deserializes String or i64 to NaiveDateTime
    struct NaiveDateTimeVisitor;

    impl<'de> de::Visitor<'de> for OptionNaiveDateTimeVisitor {
        type Value = Option<chrono::NaiveDateTime>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a null, or a DateTime string")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, d: D) -> Result<Option<chrono::NaiveDateTime>, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            Ok(d.deserialize_str(NaiveDateTimeVisitor).ok())
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where E: de::Error,
        {
            let DATETIME_FORMAT = pick_datetime_format(&s);
            Ok(chrono::NaiveDateTime::parse_from_str(s, DATETIME_FORMAT).ok())
        }

        fn visit_string<E>(self, s: String) -> Result<Self::Value, E>
            where E: de::Error,
        {
            let DATETIME_FORMAT = pick_datetime_format(&s);
            Ok(chrono::NaiveDateTime::parse_from_str(&s, DATETIME_FORMAT).ok())
        }
    }

    impl<'de> de::Visitor<'de> for NaiveDateTimeVisitor {
        type Value = chrono::NaiveDateTime;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a DateTime string in RFC3339 format, like: <2019-03-26T08:15:29Z>")
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where E: de::Error,
        {
            let DATETIME_FORMAT = pick_datetime_format(&s);
            match chrono::NaiveDateTime::parse_from_str(s, DATETIME_FORMAT) {
                Ok(d) => Ok(d),
                Err(e) => Err(E::custom(format!("Parse error {} for {}", e, s))),
            }
        }

        fn visit_string<E>(self, s: String) -> Result<Self::Value, E>
            where E: de::Error,
        {
            let DATETIME_FORMAT = pick_datetime_format(&s);
            match chrono::NaiveDateTime::parse_from_str(&s, DATETIME_FORMAT) {
                Ok(d) => Ok(d),
                Err(e) => Err(E::custom(format!("Parse error {} for {}", e, s))),
            }
        }
    }
    // use our visitor to deserialize
    deserializer.deserialize_any(OptionNaiveDateTimeVisitor)
}



// "%Y-%m-%dT%H:%M:%SZ" => chrono::NaiveDateTime
pub fn from_datetimestr_to_naivedatetime<'de, D>(
    deserializer: D,
) -> Result<chrono::NaiveDateTime, D::Error>
where
    D: de::Deserializer<'de>,
{
    // define a visitor that deserializes String or i64 to NaiveDateTime
    struct NaiveDateTimeVisitor;

    impl<'de> de::Visitor<'de> for NaiveDateTimeVisitor {
        type Value = chrono::NaiveDateTime;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string in: <2019-05-18 11:25:01> format")
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where E: de::Error,
        {
            let DATETIME_FORMAT = pick_datetime_format(&s);
            Ok(chrono::NaiveDateTime::parse_from_str(&s, DATETIME_FORMAT)
                .expect("Serde deserialize error with NaiveDateTime"))
        }

        fn visit_string<E>(self, s: String) -> Result<Self::Value, E>
            where E: de::Error,
        {
            let DATETIME_FORMAT = pick_datetime_format(&s);
            Ok(chrono::NaiveDateTime::parse_from_str(&s, DATETIME_FORMAT)
                .expect("Serde deserialize error with NaiveDateTime"))
        }
    }
    // use our visitor to deserialize
    deserializer.deserialize_any(NaiveDateTimeVisitor)
}

pub fn from_timestamp_ms_to_naivedatetime<'de, D>(
    deserializer: D,
) -> Result<chrono::NaiveDateTime, D::Error>
where
    D: de::Deserializer<'de>,
{
    // define a visitor that deserializes String or i64 to NaiveDateTime
    struct NaiveDateTimeVisitor;

    impl<'de> de::Visitor<'de> for NaiveDateTimeVisitor {
        type Value = chrono::NaiveDateTime;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string, i64, u64, or f64 containing timestamp data.")
        }

        fn visit_i64<E>(self, timestamp: i64) -> Result<Self::Value, E>
            where E: de::Error,
        {
            Ok(create_timestamp_ms(timestamp))
        }

        fn visit_u64<E>(self, timestamp: u64) -> Result<Self::Value, E>
            where E: de::Error,
        {
            Ok(create_timestamp_ms(timestamp))
        }

        fn visit_f64<E>(self, timestamp: f64) -> Result<Self::Value, E>
            where E: de::Error,
        {
            Ok(create_timestamp_ms(timestamp))
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where E: de::Error,
        {
            let timestamp = s.parse::<i64>().unwrap();
            Ok(create_timestamp_ms(timestamp))
        }

        fn visit_string<E>(self, s: String) -> Result<Self::Value, E>
            where E: de::Error,
        {
            let timestamp = s.parse::<i64>().unwrap();
            Ok(create_timestamp_ms(timestamp))
        }
    }
    // use our visitor to deserialize
    deserializer.deserialize_any(NaiveDateTimeVisitor)
}


fn create_timestamp_ms<T>(_timestamp: T) -> chrono::NaiveDateTime
where
    T: Div + ToPrimitive + AddAssign + Default,
{
    // convert to NaiveDateTime
    let timestamp: i64 = _timestamp.to_i64().unwrap();
    // Timestamps should much much larger than 10 digits
    match timestamp {
        0..=1_000_000_000_000 => {
            // Timestamp of 1_000_000_000_000 is 9th Sep, 2001, 01:46:40
            warn!("Timestamp too small: {}.\n
            Timestamp format may be in seconds instead of milliseconds", timestamp);
            let ms = (timestamp % 1000) * 1_000_000;
            // get remainder in milliseconds, convert to nanoseconds
            // as from_timestamp takes nanoseconds in the 2nd argument
            chrono::NaiveDateTime::from_timestamp(timestamp / 1_000, ms as u32)
            // first argument is seconds, second argument is in nanoseconds
        }
        _ => {
            // Timestamp are in milliseconds
            let ms = (timestamp % 1000) * 1_000_000;
            // get remainder in milliseconds, convert to nanoseconds
            // as from_timestamp takes nanoseconds in the 2nd argument
            chrono::NaiveDateTime::from_timestamp(timestamp / 1_000, ms as u32)
            // first argument is seconds, second argument is in nanoseconds
        }
    }
}

pub fn from_timestamp_s_to_naivedatetime<'de, D>(
    deserializer: D,
) -> Result<chrono::NaiveDateTime, D::Error>
where
    D: de::Deserializer<'de>,
{
    // define a visitor that deserializes String or i64 to NaiveDateTime
    struct NaiveDateTimeVisitor;

    impl<'de> de::Visitor<'de> for NaiveDateTimeVisitor {
        type Value = chrono::NaiveDateTime;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string, i64, u64, or f64 containing timestamp data.")
        }

        fn visit_i64<E>(self, timestamp: i64) -> Result<Self::Value, E>
            where E: de::Error,
        {
            Ok(create_timestamp_s(timestamp))
        }

        fn visit_u64<E>(self, timestamp: u64) -> Result<Self::Value, E>
            where E: de::Error,
        {
            Ok(create_timestamp_s(timestamp))
        }

        fn visit_f64<E>(self, timestamp: f64) -> Result<Self::Value, E>
            where E: de::Error,
        {
            Ok(create_timestamp_s(timestamp))
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where E: de::Error,
        {
            let timestamp = s.parse::<i64>().unwrap();
            Ok(create_timestamp_s(timestamp))
        }

        fn visit_string<E>(self, s: String) -> Result<Self::Value, E>
            where E: de::Error,
        {
            let timestamp = s.parse::<i64>().unwrap();
            Ok(create_timestamp_s(timestamp))
        }
    }
    // use our visitor to deserialize
    deserializer.deserialize_any(NaiveDateTimeVisitor)
}

fn create_timestamp_s<T>(_timestamp: T) -> chrono::NaiveDateTime
where
    T: Div + ToPrimitive + AddAssign + Default,
{
    let timestamp: i64 = _timestamp.to_i64().unwrap();
    chrono::NaiveDateTime::from_timestamp(timestamp, 0)
    // first argument is seconds, second argument is in nanoseconds
}


/////////////////////////////////
/////// Tests
////////////////////////////////

mod tests {
    use super::*;

    ///////////////////////////////////////
    //// from_datetimestr_to_naivedatetime
    ///////////////////////////////////////

    #[allow(dead_code)]
    #[derive(Deserialize)]
    struct MockDatetimeStr {
        #[serde(deserialize_with = "from_datetimestr_to_naivedatetime")]
        expires_at: chrono::NaiveDateTime,
    }

    #[test]
    fn datetimestr_with_space() {
        let DATETIME_FORMAT = "%Y-%m-%d %H:%M:%S";
        let t_benchmark = chrono::NaiveDateTime::parse_from_str(
            "2019-04-14 01:12:33", DATETIME_FORMAT
        ).unwrap();

        let test = r#"{ "expires_at": "2019-04-14 01:12:33" }"#;
        let t2: MockDatetimeStr = serde_json::from_str::<MockDatetimeStr>(&test).unwrap();
        assert_eq!(t_benchmark, t2.expires_at)
    }

    #[test]
    fn datetimestr_with_milliseconds() {
        let DATETIME_FORMAT = "%Y-%m-%d %H:%M:%S%.f";
        let t_benchmark = chrono::NaiveDateTime::parse_from_str(
            "2019-04-14 01:12:33.023", DATETIME_FORMAT
        ).unwrap();

        let test = r#"{ "expires_at": "2019-04-14 01:12:33.023" }"#;
        let t2: MockDatetimeStr = serde_json::from_str::<MockDatetimeStr>(&test).unwrap();
        assert_eq!(t_benchmark, t2.expires_at)
    }

    #[test]
    fn datetimestr_with_milliseconds_Z() {
        let DATETIME_FORMAT = "%Y-%m-%d %H:%M:%S%.fZ";
        let t_benchmark = chrono::NaiveDateTime::parse_from_str(
            "2019-04-14 01:12:33.023Z", DATETIME_FORMAT
        ).unwrap();

        let test = r#"{ "expires_at": "2019-04-14 01:12:33.023Z" }"#;
        let t2: MockDatetimeStr = serde_json::from_str::<MockDatetimeStr>(&test).unwrap();
        assert_eq!(t_benchmark, t2.expires_at)
    }

    #[test]
    fn datetimestr_with_T() {
        let DATETIME_FORMAT = "%Y-%m-%dT%H:%M:%S";
        let t_benchmark = chrono::NaiveDateTime::parse_from_str(
            "2019-04-14T01:12:33", DATETIME_FORMAT
        ).unwrap();

        let test = r#"{ "expires_at": "2019-04-14T01:12:33" }"#;
        let t2: MockDatetimeStr = serde_json::from_str::<MockDatetimeStr>(&test).unwrap();
        assert_eq!(t_benchmark, t2.expires_at)
    }

    #[test]
    fn datetimestr_with_T_and_space() {
        let DATETIME_FORMAT = "%Y-%m-%d %H:%M:%S";
        let t_benchmark = chrono::NaiveDateTime::parse_from_str(
            "2019-04-14 01:12:33", DATETIME_FORMAT
        ).unwrap();

        let test = r#"{ "expires_at": "2019-04-14T01:12:33" }"#;
        let t2: MockDatetimeStr = serde_json::from_str::<MockDatetimeStr>(&test).unwrap();
        assert_eq!(t_benchmark, t2.expires_at)
    }

    ///////////////////////////////////////
    //// from_timestamp_ms_to_naivedatetime
    ///////////////////////////////////////

    #[allow(dead_code)]
    fn create_timestamp_benchmark(sec: i64) -> chrono::NaiveDateTime {
        let ms = (sec % 1000) * 1_000_000;
        let t_benchmark =
            chrono::NaiveDateTime::from_timestamp(sec as i64 / 1_000 as i64, ms as u32);
        t_benchmark
    }

    #[allow(dead_code)]
    #[derive(Deserialize)]
    struct MockJsonTimestamp {
        #[serde(deserialize_with = "from_timestamp_ms_to_naivedatetime")]
        json_time: chrono::NaiveDateTime,
    }

    #[test]
    fn test_mock_timestamp_struct() {
        let timestamp = 1_222_333_444_555;
        let ms = (timestamp % 1000) * 1_000_000;
        let t1: MockJsonTimestamp = MockJsonTimestamp {
            json_time: chrono::NaiveDateTime::from_timestamp(timestamp / 1_000, ms as u32)
        };
        let t_benchmark = create_timestamp_benchmark(timestamp);
        assert_eq!(t1.json_time, t_benchmark)
    }

    #[test]
    fn try_naive_date_time_str() {
        let _t1 = r#"{ "json_time": "1222333444555" }"#;
        let t1: MockJsonTimestamp = serde_json::from_str::<MockJsonTimestamp>(&_t1).unwrap();
        let t_benchmark = create_timestamp_benchmark(1_222_333_444_555);
        assert_eq!(t1.json_time, t_benchmark)
    }

    #[test]
    fn try_naive_date_time_int() {
        let _t1 = r#"{ "json_time": 1222333444555 }"#;
        let t1: MockJsonTimestamp = serde_json::from_str::<MockJsonTimestamp>(&_t1).unwrap();
        let t_benchmark = create_timestamp_benchmark(1_222_333_444_555);
        assert_eq!(t1.json_time, t_benchmark)
    }

    #[test]
    fn try_naive_date_time_float() {
        let _t1 = r#"{ "json_time": 1222333444555.0 }"#;
        let t1: MockJsonTimestamp = serde_json::from_str::<MockJsonTimestamp>(&_t1).unwrap();
        let t_benchmark = create_timestamp_benchmark(1_222_333_444_555);
        assert_eq!(t1.json_time, t_benchmark)
    }

}
