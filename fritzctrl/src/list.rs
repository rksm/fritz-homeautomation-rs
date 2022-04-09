use clap::ArgMatches;
use log::info;
use prettytable::{format, Cell, Row, Table};
use std::collections::HashSet;

pub(crate) fn list(args: &ArgMatches) -> anyhow::Result<()> {
    let user = args.value_of("user").unwrap();
    let password = args.value_of("password").unwrap();
    let ain = args.value_of("device");
    let kinds = args.value_of("kinds").map(|kinds| {
        crate::parser::parse_kinds(kinds)
            .unwrap_or_default()
            .into_iter()
            .collect()
    });
    let limit = args
        .value_of("limit")
        .map(|limit| limit.parse().unwrap_or_default());

    let mut client = fritzapi::FritzClient::new(user, password);
    let devices = client.list_devices()?;

    if let Some(ain) = ain {
        let device = match devices.into_iter().find(|dev| dev.id() == ain) {
            None => {
                return Err(anyhow::anyhow!("Cannot find device with ain {:?}", ain));
            }
            Some(device) => device,
        };

        let tables = device_detail_table(&mut client, &device, &kinds, limit)?
            .into_iter()
            .map(|ea| ea.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        if kinds.is_none() {
            print_device_table(&[device]);
            println!();
        }
        print!("{}", tables);

        return Ok(());
    }

    info!("found {} devices", devices.len());
    print_device_table(&devices);

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

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

fn print_device_table(devices: &[fritzapi::AVMDevice]) {
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
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

fn device_detail_table(
    client: &mut fritzapi::FritzClient,
    device: &fritzapi::AVMDevice,
    kinds: &Option<HashSet<fritzapi::DeviceStatsKind>>,
    limit: Option<usize>,
) -> anyhow::Result<Vec<Table>> {
    client
        .device_stats(device.id())?
        .into_iter()
        .filter_map(|stat| {
            match kinds {
                Some(kinds) if !kinds.contains(&stat.kind) => return None,
                _ => {}
            }
            let mut table = create_table();
            table.set_titles(Row::new(vec![
                Cell::new_align("time", format::Alignment::CENTER),
                Cell::new_align(
                    &format!("{:?} ({})", stat.kind, stat.kind.unit()),
                    format::Alignment::CENTER,
                ),
            ]));
            print_stat(&mut table, &stat, limit);
            Some(Ok(table))
        })
        .collect()
}

fn print_stat(table: &mut Table, stat: &fritzapi::DeviceStats, limit: Option<usize>) {
    let now = chrono::Local::now();
    for values in &stat.values {
        let mut n = 0;
        let mut time = now;
        for val in &values.values {
            table.add_row(Row::new(vec![
                Cell::new(&time.format("%Y-%m-%d %H:%M:%S").to_string()),
                Cell::new_align(&format!("{:.1}", val), format::Alignment::RIGHT),
            ]));
            time = time - chrono::Duration::seconds(values.grid as i64);
            n += 1;
            match limit {
                Some(limit) if n > limit => break,
                _ => continue,
            }
        }
    }
}
