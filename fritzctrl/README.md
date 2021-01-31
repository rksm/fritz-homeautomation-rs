# fritzctrl

Small Rust project to inspect and control [FRITZ!DECT](https://avm.de/produkte/fritzdect/) devices.

### Usage

The command line tool has several subcommands:
- list: List all devices or list sensor data of individual device.
- switch: Turn device on / off.
- schedule: Reads and parses lines from stdin that contain date, device id, and state. Runs until all commands are processed.
- daylight: Helper command that prints sunrise / sunset times for a given location and time range.

Pretty much all commands need the fritz.box user name and password. You can set it in an env vars `FRTIZ_USER` and `FRITZ_PASSWORD` or pass it as arguments to the subcommands (the user / password combo is the same you use for http://fritz.box).

### Examples

#### List all devices

`$ fritzctrl list --user xxx --password yyy`

```
      id       |    product     |            name             | state
---------------+----------------+-----------------------------+-------
 11630 0069103 | FRITZ!DECT 200 | FRITZ!DECT 200 Laufband     | on
 11657 0272633 | FRITZ!DECT 210 | FRITZ!DECT 210 #2           | off
 11630 0128064 | FRITZ!DECT 200 | FRITZ!DECT 200 Schreibtisch | on
 09995 0335100 | FRITZ!DECT 301 | FRITZ!DECT 301 #4           |
 11630 0123723 | FRITZ!DECT 200 | FRITZ!DECT 200 #5           | off
```

#### List last 5 temperature readings of one device

`$ fritzctrl list --device "11630 0123723" --kinds temp --limit 3`

```
        time         | Temperature (°C)
---------------------+------------------
 2021-01-31 23:42:31 |             22.0
 2021-01-31 23:27:31 |             23.0
 2021-01-31 23:12:31 |             23.0
 2021-01-31 22:57:31 |             23.0
```

#### Turn device on

`$ fritzctrl switch --device "11630 0123723" --on`


#### Schedule switching a device based on daylight hours

1. First figure out what the times you want to turn the device on / off are. E.g.
`$ fritzctrl daylight --from-date 2021-02-01 --to-date 2021-02-03 --shift-from="-30min" --shift-to="30hour"`
generates sunrise / sunset times shifted by -30 minutes (sunrise) and +30 minutes sunset:

```
using device location (_, _)
sunrise: 2021-02-01 07:17:57
sunset: 2021-02-01 17:20:41
sunrise: 2021-02-02 07:16:20
sunset: 2021-02-02 17:22:36
sunrise: 2021-02-03 07:14:40
sunset: 2021-02-03 17:24:30
```

Then store some commands into a file:

`fritz-commands.txt`:

```
2021-02-01 06:00:00 11630 0123723 on
2021-02-01 07:17:57 11630 0123723 off
2021-02-01 17:20:41 11630 0123723 on
2021-02-01 22:30:00 11630 0123723 off
```

You can run start processing those commands with
`$ cat fritz-commands.txt | fritzctrl schedule`

The program will wait until the next command should run and then toggle the device state. Once all commands are done the app will exit.

### Why???

Useful for scheduling your Christmas lights!

### Fritz API

Uses the [fritz HTTP API](https://avm.de/fileadmin/user_upload/Global/Service/Schnittstellen/AHA-HTTP-Interface.pdf).

#### Rust API

If you want to integrate directly with the API have a look at the [fritzapi crate](https://crates.io/crates/fritzapi).


License: MIT
