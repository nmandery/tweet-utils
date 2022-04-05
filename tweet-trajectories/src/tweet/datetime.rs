// from tweet 0.3 crate

use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Deserializer};

const FORMAT: &str = "%a %b %e %T %z %Y";

pub fn datefmt_de<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Utc.datetime_from_str(&s, FORMAT)
        .map_err(serde::de::Error::custom)
}
