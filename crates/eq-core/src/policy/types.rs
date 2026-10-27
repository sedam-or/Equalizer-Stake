//! Canonical policy values with validated constructors and serde boundaries (spec/04 §5, CAN-10).

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};
use thiserror::Error;

use crate::money::MONEY_MAX_ABS_MINOR;

/// Validation failures for common policy values (CQ-23, CAN-10).
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PolicyTypeError {
    /// A value failed the invariant named by the static field label.
    #[error("invalid {0}")]
    Invalid(&'static str),
}

/// Canonical percentage in basis points, bounded to 0 through 10000 (ARC-11, CAN-10).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, utoipa::ToSchema)]
#[serde(try_from = "u32")]
#[schemars(with = "BpsSchema")]
pub struct Bps(#[schemars(range(min = 0, max = 10_000))] u32);

#[derive(JsonSchema)]
#[schemars(rename = "Bps", transparent)]
#[expect(
    dead_code,
    reason = "Schemars reads this helper as the bounded Bps schema (CAN-10)"
)]
struct BpsSchema(#[schemars(range(min = 0, max = 10_000))] u32);

impl Bps {
    /// Zero percent, represented as zero basis points (ARC-11).
    pub const ZERO: Self = Self(0);
    /// One hundred percent, represented as ten thousand basis points (ARC-11).
    pub const FULL: Self = Self(10_000);

    /// Validates basis points in the canonical range 0 through 10000 (CAN-10).
    ///
    /// # Errors
    /// Returns [`PolicyTypeError::Invalid`] when the value exceeds 10000.
    pub fn new(value: u32) -> Result<Self, PolicyTypeError> {
        if value > 10_000 {
            return Err(PolicyTypeError::Invalid("basis points"));
        }
        Ok(Self(value))
    }

    /// Creates a basis-point constant and fails compilation when its literal is out of range (CQ-23).
    pub const fn from_const(value: u32) -> Self {
        assert!(value <= 10_000, "basis points out of range");
        Self(value)
    }

    /// Returns the basis-point value (ARC-11).
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl TryFrom<u32> for Bps {
    type Error = PolicyTypeError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// Non-negative policy amount in minor units of the tenant currency (POL-12, CAN-10).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(try_from = "i64")]
#[schemars(with = "MinorAmountSchema")]
pub struct MinorAmount(#[schemars(range(min = 0, max = 1_000_000_000_000_000_i64))] i64);

#[derive(JsonSchema)]
#[schemars(rename = "MinorAmount", transparent)]
#[expect(
    dead_code,
    reason = "Schemars reads this helper as the bounded MinorAmount schema (CAN-10)"
)]
struct MinorAmountSchema(#[schemars(range(min = 0, max = 1_000_000_000_000_000_i64))] i64);

impl MinorAmount {
    /// Validates a non-negative amount within the canonical money cap (POL-12, CAN-10).
    ///
    /// # Errors
    /// Returns [`PolicyTypeError::Invalid`] for a negative amount or one above the cap.
    pub fn new(value: i64) -> Result<Self, PolicyTypeError> {
        if !(0..=MONEY_MAX_ABS_MINOR).contains(&value) {
            return Err(PolicyTypeError::Invalid("minor amount"));
        }
        Ok(Self(value))
    }

    /// Creates a policy amount constant and fails compilation when its literal is out of range (CQ-23).
    pub const fn from_const(value: i64) -> Self {
        assert!(
            value >= 0 && value <= MONEY_MAX_ABS_MINOR,
            "minor amount out of range"
        );
        Self(value)
    }

    /// Returns the amount in minor units (POL-12).
    pub const fn get(self) -> i64 {
        self.0
    }
}

impl TryFrom<i64> for MinorAmount {
    type Error = PolicyTypeError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// Day number valid in every calendar month (POL-11, CAN-10).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(try_from = "u8")]
#[schemars(with = "DayOfMonthSchema")]
pub struct DayOfMonth(#[schemars(range(min = 1, max = 28))] u8);

#[derive(JsonSchema)]
#[schemars(rename = "DayOfMonth", transparent)]
#[expect(
    dead_code,
    reason = "Schemars reads this helper as the bounded DayOfMonth schema (POL-11)"
)]
struct DayOfMonthSchema(#[schemars(range(min = 1, max = 28))] u8);

impl DayOfMonth {
    /// Validates a day number from 1 through 28 (POL-11).
    ///
    /// # Errors
    /// Returns [`PolicyTypeError::Invalid`] outside the inclusive range 1 through 28.
    pub fn new(value: u8) -> Result<Self, PolicyTypeError> {
        if !(1..=28).contains(&value) {
            return Err(PolicyTypeError::Invalid("day of month"));
        }
        Ok(Self(value))
    }

    /// Creates a day-of-month constant and fails compilation when its literal is out of range (CQ-23).
    pub const fn from_const(value: u8) -> Self {
        assert!(value >= 1 && value <= 28, "day of month out of range");
        Self(value)
    }

    /// Returns the day number (POL-11).
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for DayOfMonth {
    type Error = PolicyTypeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// Language supported by v1 policy text (CAN-10).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    /// English.
    En,
    /// Indonesian.
    Id,
}

/// Weekday encoded as its lowercase three-letter ISO calendar form (CAN-10).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Weekday {
    /// Monday.
    Mon,
    /// Tuesday.
    Tue,
    /// Wednesday.
    Wed,
    /// Thursday.
    Thu,
    /// Friday.
    Fri,
    /// Saturday.
    Sat,
    /// Sunday.
    Sun,
}

impl Weekday {
    const ORDERED: [Self; 7] = [
        Self::Mon,
        Self::Tue,
        Self::Wed,
        Self::Thu,
        Self::Fri,
        Self::Sat,
        Self::Sun,
    ];
}

/// A local time with minute precision, stored as minutes after midnight (CAN-10).
#[derive(Clone, Copy, Debug, PartialEq, Eq, JsonSchema)]
#[schemars(with = "LocalTimeSchema")]
#[serde(try_from = "String")]
pub struct LocalTime(u16);

#[derive(JsonSchema)]
#[schemars(rename = "LocalTime", transparent)]
#[expect(
    dead_code,
    reason = "Schemars reads this helper as the canonical LocalTime schema (CAN-10)"
)]
struct LocalTimeSchema(#[schemars(regex(pattern = r"^(?:[01][0-9]|2[0-3]):[0-5][0-9]$"))] String);

impl LocalTime {
    /// Parses a canonical `HH:MM` time or an accepted `HH:MM:00` input (CAN-10).
    ///
    /// # Errors
    /// Returns [`PolicyTypeError::Invalid`] for malformed values or times outside 00:00 through 23:59.
    pub fn new(value: &str) -> Result<Self, PolicyTypeError> {
        parse_time_minutes(value, false).map(Self)
    }

    /// Creates a local-time constant and fails compilation for values after 23:59 (CQ-23, CAN-10).
    pub const fn from_const(minutes_of_day: u16) -> Self {
        assert!(minutes_of_day < 1_440, "local time out of range");
        Self(minutes_of_day)
    }

    /// Returns minutes since midnight in the range 0 through 1439 (CAN-10).
    pub const fn minutes_since_midnight(self) -> u16 {
        self.0
    }

    /// Returns the zero-padded canonical time string (CAN-10).
    pub fn as_hh_mm(self) -> String {
        format!("{:02}:{:02}", self.0 / 60, self.0 % 60)
    }
}

impl TryFrom<String> for LocalTime {
    type Error = PolicyTypeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl Serialize for LocalTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.as_hh_mm())
    }
}

impl<'de> Deserialize<'de> for LocalTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_from(String::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

fn parse_time_minutes(value: &str, allow_end_of_day: bool) -> Result<u16, PolicyTypeError> {
    let time = if value.len() == 5 {
        value
    } else if value.len() == 8 && value.ends_with(":00") {
        &value[..5]
    } else {
        return Err(PolicyTypeError::Invalid("local time"));
    };
    let bytes = time.as_bytes();
    if bytes.get(2) != Some(&b':')
        || !bytes.first().is_some_and(u8::is_ascii_digit)
        || !bytes.get(1).is_some_and(u8::is_ascii_digit)
        || !bytes.get(3).is_some_and(u8::is_ascii_digit)
        || !bytes.get(4).is_some_and(u8::is_ascii_digit)
    {
        return Err(PolicyTypeError::Invalid("local time"));
    }
    let hour = u16::from(bytes[0] - b'0') * 10 + u16::from(bytes[1] - b'0');
    let minute = u16::from(bytes[3] - b'0') * 10 + u16::from(bytes[4] - b'0');
    if hour == 24 && minute == 0 && allow_end_of_day {
        return Ok(1_440);
    }
    if hour > 23 || minute > 59 {
        return Err(PolicyTypeError::Invalid("local time"));
    }
    Ok(hour * 60 + minute)
}

/// Half-open same-day interval; 24:00 is valid only as its end (spec/04 §5, CAN-10).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(try_from = "TimeIntervalRepr")]
#[schemars(with = "TimeIntervalSchema")]
pub struct TimeInterval {
    start: LocalTime,
    end_minutes: u16,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TimeIntervalRepr {
    start: String,
    end: String,
}

#[derive(Serialize)]
struct TimeIntervalSerialize {
    start: LocalTime,
    end: String,
}

#[derive(JsonSchema)]
#[schemars(rename = "TimeInterval")]
#[expect(
    dead_code,
    reason = "Schemars reads this helper as the context-aware TimeInterval schema (CAN-10)"
)]
struct TimeIntervalSchema {
    #[schemars(regex(pattern = r"^(?:[01][0-9]|2[0-3]):[0-5][0-9]$"))]
    start: String,
    #[schemars(regex(pattern = r"^(?:(?:[01][0-9]|2[0-3]):[0-5][0-9]|24:00)$"))]
    end: String,
}

impl TimeInterval {
    /// Validates a same-day interval with a strictly later end (spec/04 §5).
    ///
    /// # Errors
    /// Returns [`PolicyTypeError::Invalid`] if start is not before end.
    pub fn new(start: LocalTime, end: LocalTime) -> Result<Self, PolicyTypeError> {
        Self::from_end_minutes(start, end.0)
    }

    /// Creates a validated interval ending at midnight (24:00), represented only in this interval (CAN-10).
    ///
    /// # Errors
    /// Returns [`PolicyTypeError::Invalid`] when the start cannot precede the end of day.
    pub fn until_midnight(start: LocalTime) -> Result<Self, PolicyTypeError> {
        Self::from_end_minutes(start, 1_440)
    }

    fn from_end_minutes(start: LocalTime, end_minutes: u16) -> Result<Self, PolicyTypeError> {
        if start.0 >= end_minutes {
            return Err(PolicyTypeError::Invalid("time interval"));
        }
        Ok(Self { start, end_minutes })
    }

    /// Returns the interval start (spec/04 §5).
    pub const fn start(self) -> LocalTime {
        self.start
    }

    /// Returns the interval end in canonical `HH:MM` form, including `24:00` at midnight (CAN-10).
    pub fn end_hh_mm(self) -> String {
        if self.end_minutes == 1_440 {
            String::from("24:00")
        } else {
            format!("{:02}:{:02}", self.end_minutes / 60, self.end_minutes % 60)
        }
    }
}

impl Serialize for TimeInterval {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        TimeIntervalSerialize {
            start: self.start,
            end: self.end_hh_mm(),
        }
        .serialize(serializer)
    }
}

impl TryFrom<TimeIntervalRepr> for TimeInterval {
    type Error = PolicyTypeError;

    fn try_from(value: TimeIntervalRepr) -> Result<Self, Self::Error> {
        let start = LocalTime(parse_time_minutes(&value.start, false)?);
        let end_minutes = parse_time_minutes(&value.end, true)?;
        Self::from_end_minutes(start, end_minutes)
    }
}

/// Opening-hours value for one weekday (spec/04 §5).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(try_from = "DayHoursRepr")]
pub struct DayHours {
    weekday: Weekday,
    open: bool,
    intervals: Vec<TimeInterval>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct DayHoursRepr {
    weekday: Weekday,
    open: bool,
    intervals: Vec<TimeInterval>,
}

impl DayHours {
    /// Validates sorted, non-overlapping intervals and the closed-day invariant (spec/04 §5).
    ///
    /// # Errors
    /// Returns [`PolicyTypeError::Invalid`] for overlapping/unsorted intervals or intervals on a closed day.
    pub fn new(weekday: Weekday, open: bool, intervals: Vec<TimeInterval>) -> Result<Self, PolicyTypeError> {
        if (!open && !intervals.is_empty())
            || intervals
                .windows(2)
                .any(|pair| pair[0].end_minutes > pair[1].start.0)
        {
            return Err(PolicyTypeError::Invalid("day hours"));
        }
        Ok(Self {
            weekday,
            open,
            intervals,
        })
    }

    /// Returns the weekday associated with these hours (spec/04 §5).
    pub const fn weekday(&self) -> Weekday {
        self.weekday
    }

    /// Returns whether this day is open (spec/04 §5).
    pub const fn is_open(&self) -> bool {
        self.open
    }

    /// Returns the sorted intervals for this weekday (spec/04 §5).
    pub fn intervals(&self) -> &[TimeInterval] {
        &self.intervals
    }
}

impl TryFrom<DayHoursRepr> for DayHours {
    type Error = PolicyTypeError;

    fn try_from(value: DayHoursRepr) -> Result<Self, Self::Error> {
        Self::new(value.weekday, value.open, value.intervals)
    }
}

/// Seven validated opening-hours entries in Monday-through-Sunday order (spec/04 §5).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(try_from = "Vec<DayHours>")]
#[schemars(with = "WeeklyHoursSchema")]
pub struct WeeklyHours(#[schemars(length(min = 7, max = 7))] [DayHours; 7]);

#[derive(JsonSchema)]
#[schemars(rename = "WeeklyHours", transparent)]
#[expect(
    dead_code,
    reason = "Schemars reads this helper as the fixed-length WeeklyHours schema (spec/04 §5)"
)]
struct WeeklyHoursSchema(#[schemars(length(min = 7, max = 7))] [DayHours; 7]);

impl WeeklyHours {
    /// Validates exactly seven entries ordered Monday through Sunday (spec/04 §5).
    ///
    /// # Errors
    /// Returns [`PolicyTypeError::Invalid`] when the weekdays are not in canonical order.
    pub fn new(days: [DayHours; 7]) -> Result<Self, PolicyTypeError> {
        if !days
            .iter()
            .map(DayHours::weekday)
            .zip(Weekday::ORDERED)
            .all(|(actual, expected)| actual == expected)
        {
            return Err(PolicyTypeError::Invalid("weekly hours"));
        }
        Ok(Self(days))
    }

    /// Returns the Monday-through-Sunday day entries (spec/04 §5).
    pub const fn days(&self) -> &[DayHours; 7] {
        &self.0
    }
}

impl TryFrom<Vec<DayHours>> for WeeklyHours {
    type Error = PolicyTypeError;

    fn try_from(days: Vec<DayHours>) -> Result<Self, Self::Error> {
        let days: [DayHours; 7] = days
            .try_into()
            .map_err(|_| PolicyTypeError::Invalid("weekly hours"))?;
        Self::new(days)
    }
}

/// Localized text keyed by supported locale, with at least one entry (spec/04 §5).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(try_from = "BTreeMap<Locale, String>")]
#[schemars(with = "I18nTextSchema")]
pub struct I18nText(#[schemars(length(min = 1))] BTreeMap<Locale, String>);

#[derive(JsonSchema)]
#[schemars(rename = "I18nText", transparent)]
#[schemars(extend("minProperties" = 1))]
#[expect(
    dead_code,
    reason = "Schemars reads this helper as the non-empty I18nText schema (spec/04 §5)"
)]
struct I18nTextSchema(BTreeMap<Locale, String>);

impl I18nText {
    /// Validates a non-empty localized text map (spec/04 §5).
    ///
    /// # Errors
    /// Returns [`PolicyTypeError::Invalid`] when no locale entry exists.
    pub fn new(values: BTreeMap<Locale, String>) -> Result<Self, PolicyTypeError> {
        if values.is_empty() {
            return Err(PolicyTypeError::Invalid("i18n text"));
        }
        Ok(Self(values))
    }

    /// Returns text for a requested locale when present (spec/04 §5).
    pub fn get(&self, locale: &Locale) -> Option<&str> {
        self.0.get(locale).map(String::as_str)
    }

    /// Returns all supported localized values (spec/04 §5).
    pub fn values(&self) -> &BTreeMap<Locale, String> {
        &self.0
    }
}

impl TryFrom<BTreeMap<Locale, String>> for I18nText {
    type Error = PolicyTypeError;

    fn try_from(values: BTreeMap<Locale, String>) -> Result<Self, Self::Error> {
        Self::new(values)
    }
}

/// Geographic point with finite coordinates inside WGS 84 latitude/longitude bounds (spec/04 §5).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(try_from = "GeoPointRepr")]
pub struct GeoPoint {
    #[schemars(range(min = -90.0, max = 90.0))]
    lat: f64,
    #[schemars(range(min = -180.0, max = 180.0))]
    lng: f64,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct GeoPointRepr {
    #[schemars(range(min = -90.0, max = 90.0))]
    lat: f64,
    #[schemars(range(min = -180.0, max = 180.0))]
    lng: f64,
}

impl GeoPoint {
    /// Validates latitude and longitude bounds and rejects non-finite coordinates (spec/04 §5).
    ///
    /// # Errors
    /// Returns [`PolicyTypeError::Invalid`] for NaN, infinity, or out-of-range coordinates.
    pub fn new(lat: f64, lng: f64) -> Result<Self, PolicyTypeError> {
        if !lat.is_finite()
            || !lng.is_finite()
            || !(-90.0..=90.0).contains(&lat)
            || !(-180.0..=180.0).contains(&lng)
        {
            return Err(PolicyTypeError::Invalid("geo point"));
        }
        Ok(Self { lat, lng })
    }

    /// Returns latitude in degrees (spec/04 §5).
    pub const fn lat(self) -> f64 {
        self.lat
    }

    /// Returns longitude in degrees (spec/04 §5).
    pub const fn lng(self) -> f64 {
        self.lng
    }
}

impl TryFrom<GeoPointRepr> for GeoPoint {
    type Error = PolicyTypeError;

    fn try_from(value: GeoPointRepr) -> Result<Self, Self::Error> {
        Self::new(value.lat, value.lng)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use insta::assert_json_snapshot;
    use serde_json::{Value, json};

    use crate::money::Money;

    use super::{
        Bps, DayHours, DayOfMonth, GeoPoint, I18nText, LocalTime, Locale, MinorAmount, TimeInterval, Weekday,
        WeeklyHours,
    };

    fn local_time(value: &str) -> LocalTime {
        LocalTime::new(value).expect("valid local time")
    }

    #[test]
    fn day_of_month_max_28() {
        assert!(DayOfMonth::new(29).is_err());
        assert!(serde_json::from_str::<DayOfMonth>("29").is_err());
        assert_eq!(DayOfMonth::new(28).expect("day 28 is valid").get(), 28);
    }

    #[test]
    fn day_hours_rejects_overlap() {
        let first = TimeInterval::new(local_time("09:00"), local_time("10:00")).expect("valid interval");
        let second = TimeInterval::new(local_time("09:30"), local_time("11:00")).expect("valid interval");

        assert!(DayHours::new(Weekday::Mon, true, vec![first, second]).is_err());
        let wire = json!({
            "weekday": "mon",
            "open": true,
            "intervals": [
                {"start": "09:00", "end": "10:00"},
                {"start": "09:30", "end": "11:00"}
            ]
        });
        assert!(serde_json::from_value::<DayHours>(wire).is_err());
    }

    #[test]
    fn local_time_format() {
        let value = local_time("09:05");
        assert_eq!(
            serde_json::to_string(&value).expect("local time serializes"),
            "\"09:05\""
        );
        let with_seconds =
            serde_json::from_str::<LocalTime>("\"12:34:00\"").expect("zero seconds are accepted");
        assert_eq!(
            serde_json::to_string(&with_seconds).expect("local time serializes"),
            "\"12:34\""
        );
        assert!(LocalTime::new("24:00").is_err());
        assert!(LocalTime::new("12:34:01").is_err());
    }

    #[test]
    fn interval_end_2400_ok() {
        let interval =
            TimeInterval::until_midnight(local_time("23:00")).expect("24:00 is a valid interval end");
        assert_eq!(interval.end_hh_mm(), "24:00");

        assert_eq!(
            serde_json::to_value(interval).expect("interval serializes"),
            json!({"start": "23:00", "end": "24:00"})
        );
        assert!(serde_json::from_value::<TimeInterval>(json!({"start":"24:00","end":"24:00"})).is_err());
    }

    #[test]
    fn i18n_text_requires_entry() {
        assert!(I18nText::new(BTreeMap::new()).is_err());
        let values = BTreeMap::from([(Locale::En, String::from("Welcome"))]);
        assert_eq!(
            I18nText::new(values)
                .expect("one locale is valid")
                .get(&Locale::En),
            Some("Welcome")
        );
        assert!(serde_json::from_str::<I18nText>("{}").is_err());
    }

    #[test]
    fn geo_bounds() {
        assert!(GeoPoint::new(-90.0, -180.0).is_ok());
        assert!(GeoPoint::new(90.0, 180.0).is_ok());
        assert!(GeoPoint::new(-90.01, 0.0).is_err());
        assert!(GeoPoint::new(0.0, 180.01).is_err());
        assert!(GeoPoint::new(f64::NAN, 0.0).is_err());
    }

    #[test]
    fn constrained_types_reject_invalid() {
        assert!(Bps::new(10_001).is_err());
        assert!(serde_json::from_str::<Bps>("10001").is_err());
        assert!(MinorAmount::new(-1).is_err());
        assert!(serde_json::from_str::<MinorAmount>("-1").is_err());
        assert!(DayOfMonth::new(29).is_err());
        assert!(GeoPoint::new(f64::NAN, 0.0).is_err());
        assert!(I18nText::new(BTreeMap::new()).is_err());

        let overlapping = json!({
            "weekday":"mon", "open":true,
            "intervals":[{"start":"09:00","end":"10:00"},{"start":"09:30","end":"11:00"}]
        });
        assert!(serde_json::from_value::<DayHours>(overlapping).is_err());

        let unordered_week = json!([
            {"weekday":"tue","open":false,"intervals":[]},
            {"weekday":"mon","open":false,"intervals":[]},
            {"weekday":"wed","open":false,"intervals":[]},
            {"weekday":"thu","open":false,"intervals":[]},
            {"weekday":"fri","open":false,"intervals":[]},
            {"weekday":"sat","open":false,"intervals":[]},
            {"weekday":"sun","open":false,"intervals":[]}
        ]);
        assert!(serde_json::from_value::<WeeklyHours>(unordered_week).is_err());
    }

    #[test]
    fn weekly_hours_schema_snapshot() {
        let schema = schemars::schema_for!(WeeklyHours);
        let value = serde_json::to_value(schema).expect("schema serializes");

        fn has_seven_item_bounds(value: &Value) -> bool {
            match value {
                Value::Object(object) => {
                    (object.get("minItems") == Some(&json!(7)) && object.get("maxItems") == Some(&json!(7)))
                        || object.values().any(has_seven_item_bounds)
                }
                Value::Array(items) => items.iter().any(has_seven_item_bounds),
                _ => false,
            }
        }

        assert!(has_seven_item_bounds(&value));
        assert_json_snapshot!(value);
    }

    #[test]
    fn constrained_schema_bounds() {
        fn schema_value<T: schemars::JsonSchema>() -> Value {
            serde_json::to_value(schemars::schema_for!(T)).expect("schema serializes")
        }

        fn has_pairs(value: &Value, pairs: &[(&str, Value)]) -> bool {
            match value {
                Value::Object(object) => {
                    pairs
                        .iter()
                        .all(|(key, expected)| object.get(*key) == Some(expected))
                        || object.values().any(|nested| has_pairs(nested, pairs))
                }
                Value::Array(items) => items.iter().any(|nested| has_pairs(nested, pairs)),
                _ => false,
            }
        }

        fn has_key(value: &Value, key: &str) -> bool {
            match value {
                Value::Object(object) => {
                    object.contains_key(key) || object.values().any(|nested| has_key(nested, key))
                }
                Value::Array(items) => items.iter().any(|nested| has_key(nested, key)),
                _ => false,
            }
        }

        assert!(has_pairs(
            &schema_value::<Bps>(),
            &[("minimum", json!(0)), ("maximum", json!(10_000))]
        ));
        assert!(has_pairs(
            &schema_value::<MinorAmount>(),
            &[
                ("minimum", json!(0)),
                ("maximum", json!(1_000_000_000_000_000_i64))
            ]
        ));
        assert!(has_pairs(
            &schema_value::<DayOfMonth>(),
            &[("minimum", json!(1)), ("maximum", json!(28))]
        ));
        assert!(has_pairs(
            &schema_value::<GeoPoint>(),
            &[("minimum", json!(-90.0)), ("maximum", json!(90.0))]
        ));
        assert!(has_pairs(
            &schema_value::<GeoPoint>(),
            &[("minimum", json!(-180.0)), ("maximum", json!(180.0))]
        ));
        assert!(has_pairs(
            &schema_value::<I18nText>(),
            &[("minProperties", json!(1))]
        ));
        assert!(has_pairs(
            &schema_value::<Money>(),
            &[
                ("minimum", json!(-1_000_000_000_000_000_i64)),
                ("maximum", json!(1_000_000_000_000_000_i64)),
            ]
        ));
        assert!(has_key(&schema_value::<LocalTime>(), "pattern"));
        assert!(schema_value::<TimeInterval>().to_string().contains("24:00"));
    }
}
