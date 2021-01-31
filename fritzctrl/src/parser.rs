pub(crate) fn valid_coord(val: String) -> Result<(), String> {
    val.parse::<f64>()
        .map(|_| ())
        .map_err(|err| err.to_string())
}

pub(crate) fn valid_date(val: String) -> Result<(), String> {
    chrono::NaiveDate::parse_from_str(&val, "%Y-%m-%d")
        .map(|_| ())
        .map_err(|err| err.to_string())
}

pub(crate) fn valid_shift(arg: String) -> Result<(), String> {
    parse_duration(&arg)
        .map(|_| ())
        .ok_or_else(|| "Not a valid time shift".to_string())
}

pub(crate) fn parse_duration(arg: &str) -> Option<chrono::Duration> {
    let sign = arg.starts_with('-');
    let input = if sign { &arg[1..] } else { arg };
    match parse_duration::parse(input) {
        Err(err) => {
            eprintln!("{:?}", err);
            None
        }
        Ok(parsed) => chrono::Duration::from_std(parsed)
            .ok()
            .map(|val| if sign { -val } else { val }),
    }
}

pub(crate) fn valid_usize(arg: String) -> Result<(), String> {
    arg.parse::<usize>()
        .map(|_| ())
        .map_err(|_| "Not a valid usize number".to_string())
}

pub(crate) fn parse_kinds(arg: &str) -> fritzapi::Result<Vec<fritzapi::DeviceStatsKind>> {
    arg.split(',').map(|ea| ea.parse()).collect()
}

pub(crate) fn valid_kinds(arg: String) -> Result<(), String> {
    parse_kinds(&arg).map(|_| ()).map_err(|err| err.to_string())
}
