use clap::{App, Arg, ArgMatches};
use fritzapi;
use prettytable::{format, Cell, Row, Table};
use std::collections::HashSet;

pub(crate) fn list(args: &ArgMatches) -> anyhow::Result<()> {
    let user = args.value_of("user").unwrap();
    let password = args.value_of("password").unwrap();
    let ain = args.value_of("device");
    let kinds = args
        .value_of("kinds")
        .map(|kinds| crate::parser::parse_kinds(kinds).unwrap_or_default());
    let limit = args
        .value_of("limit")
        .map(|limit| limit.parse().unwrap_or_default());

    let sid = fritzapi::get_sid(&user, &password)?;
    let devices: Vec<_> = fritzapi::list_devices(&sid)?;

    if let Some(ain) = ain {
        let device = match devices.into_iter().find(|dev| dev.id() == ain) {
            None => {
                return Err(anyhow::anyhow!("Cannot find device with ain {:?}", ain));
            }
            Some(device) => device,
        };
        println!("{}", device);
        print_info(&device, &sid, kinds, limit)?;
        return Ok(());
    }

    println!("found {} devices", devices.len());

    let mut table = create_table();
    table.set_titles(Row::new(vec![
        Cell::new_align("id", format::Alignment::CENTER),
        Cell::new_align("product", format::Alignment::CENTER),
        Cell::new_align("name", format::Alignment::CENTER),
        Cell::new_align("state", format::Alignment::CENTER),
    ]));

    for device in devices {
        table.add_row(Row::new(vec![
            Cell::new(device.id()),
            Cell::new(device.productname()),
            Cell::new(device.name()),
            Cell::new(device.state()),
        ]));
    }
    table.printstd();

    Ok(())
}

fn create_table() -> Table {
    let mut table = Table::new();
    let fmt = format::FormatBuilder::new()
        .padding(1, 1)
        .separator(
            format::LinePosition::Title,
            format::LineSeparator::new('-', '+', '+', '+'),
        )
        .column_separator('|')
        .build();
    table.set_format(fmt);
    table
}

fn print_info(
    device: &fritzapi::AVMDevice,
    sid: &str,
    kinds: Option<Vec<fritzapi::DeviceStatsKind>>,
    limit: Option<usize>,
) -> anyhow::Result<()> {
    let stats = device.fetch_device_stats(&sid)?;
    let kinds = kinds.map(|val| val.into_iter().collect());
    for stat in stats {
        print_stat(&stat, &kinds, limit);
    }

    Ok(())
}

fn print_stat(
    stat: &fritzapi::DeviceStats,
    kinds: &Option<HashSet<fritzapi::DeviceStatsKind>>,
    limit: Option<usize>,
) {
    let now = chrono::Local::now();
    println!("{:?}", stat.kind);

    match kinds {
        Some(kinds) if !kinds.contains(&stat.kind) => return,
        _ => {}
    }

    for values in &stat.values {
        let mut n = 0;
        let mut time = now;
        println!("grid: {}", values.grid);
        for val in &values.values {
            println!(
                "{time} {val}{unit}",
                time = time.format("%y-%m-%d %H:%M:%S"),
                val = val,
                unit = stat.kind.unit()
            );
            time = time - chrono::Duration::seconds(values.grid as i64);
            n += 1;
            match limit {
                Some(limit) if n > limit => break,
                _ => continue,
            }
        }
    }
}
