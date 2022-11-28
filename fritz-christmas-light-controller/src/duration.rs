use chrono::Duration;
use serde::Deserialize;

pub fn duration_pretty(d: Duration) -> String {
    let mut seconds = d.num_seconds();
    let minutes = d.num_minutes();

    if minutes > 0 {
        seconds -= minutes * 60;
    }
    format!("{minutes}mins {seconds}secs")
}

pub fn duration_parse(s: &str) -> Duration {
    let mut result = Duration::zero();
    let Some((a,b))=s.trim().split_once('_') else {
return result;
};
    result = result + Duration::minutes(a.replace("mins", "").parse().unwrap_or_default());
    result = result + Duration::seconds(b.replace("secs", "").parse().unwrap_or_default());
    result
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
    Ok(duration_parse(&String::deserialize(d)?))
}
