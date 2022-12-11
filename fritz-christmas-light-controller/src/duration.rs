use chrono::Duration;
use serde::Deserialize;

use crate::error::Error;

pub fn duration_pretty(d: Duration) -> String {
    let mut seconds = d.num_seconds();
    let minutes = d.num_minutes();

    if minutes > 0 {
        seconds -= minutes * 60;
    }
    format!("{minutes}mins {seconds}secs")
}

pub fn duration_parse(s: &str) -> Result<Duration, Error> {
    let mut result = Duration::zero();

    let (a, b) = if let Some((a, b)) = s.trim().split_once(' ') {
        (a, b)
    } else {
        return Err(Error::DurationParseError(
            "unable to parse duration from {s:?}".to_string(),
        ));
    };

    result = result
        + Duration::minutes(
            a.replace("mins", "")
                .parse()
                .map_err(|_| Error::DurationParseError("Unable to parse minutes".to_string()))?,
        );

    result = result
        + Duration::seconds(
            b.replace("secs", "")
                .parse()
                .map_err(|_| Error::DurationParseError("Unable to parse seconds".to_string()))?,
        );

    Ok(result)
}

pub fn serialize<S>(arg: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&duration_pretty(*arg))
}

pub fn deserialize<'de, D>(d: D) -> Result<Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    duration_parse(&String::deserialize(d)?)
        .map_err(|err| serde::de::Error::custom(err.to_string()))
}
