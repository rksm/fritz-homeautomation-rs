pub(crate) fn valid_date(val: &str) -> Result<chrono::NaiveDate, String> {
    chrono::NaiveDate::parse_from_str(val, "%Y-%m-%d").map_err(|err| err.to_string())
}

pub(crate) fn parse_duration(arg: &str) -> Result<chrono::Duration, String> {
    let sign = arg.starts_with('-');
    let input = if sign { &arg[1..] } else { arg };
    match parse_duration::parse(input) {
        Err(err) => Err(err.to_string()),
        Ok(parsed) => chrono::Duration::from_std(parsed)
            .map(|val| if sign { -val } else { val })
            .map_err(|err| err.to_string()),
    }
}

pub(crate) fn parse_kinds(arg: &str) -> fritzapi::Result<Vec<fritzapi::DeviceStatsKind>> {
    arg.split(',').map(|ea| ea.parse()).collect()
}
