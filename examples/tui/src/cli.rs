use clap::{value_parser, Arg, Command};

pub fn parse_args() -> clap::ArgMatches {
    Command::new("uBlox TUI")
        .author(clap::crate_authors!())
        .about("Simple TUI to show PVT and ESF statuses")
        .arg_required_else_help(true)
        .arg(
            Arg::new("debug-mode")
                .value_name("debug-mode")
                .long("debug-mode")
                .action(clap::ArgAction::SetTrue)
                .help("Bypass TUI altogether and run the u-blox connection only. Useful for debugging issues with u-blox connectivity and message parsing."),
        )
        .arg(
            Arg::new("log-file")
                .value_name("log-file")
                .long("log-file")
                .action(clap::ArgAction::SetTrue)
                .help("Log to file besides showing partial logs in the TUI"),
        )
        .arg(
            Arg::new("tui-rate")
                .value_name("tui-rate")
                .long("tui-rate")
                .required(false)
                .default_value("100")
                .value_parser(value_parser!(u64))
                .help("TUI refresh rate in milliseconds"),
        )
        .arg(
            Arg::new("port")
                .value_name("port")
                .short('p')
                .long("port")
                .required(true)
                .help("Serial port to open"),
        )
        .arg(
            Arg::new("baud")
                .value_name("baud")
                .short('s')
                .long("baud")
                .required(false)
                .default_value("9600")
                .value_parser(value_parser!(u32))
                .help("Baud rate of the port to open"),
        )
        .arg(
            Arg::new("stop-bits")
                .long("stop-bits")
                .help("Number of stop bits to use for opened port")
                .required(false)
                .value_parser(["1", "2"])
                .default_value("1"),
        )
        .arg(
            Arg::new("data-bits")
                .long("data-bits")
                .help("Number of data bits to use for opened port")
                .required(false)
                .value_parser(["7", "8"])
                .default_value("8"),
        )
        .arg(
            Arg::new("parity")
                .long("parity")
                .help("Parity to use for open port")
                .required(false)
                .value_parser(["even", "odd"]),
        )
        .subcommand(
            Command::new("configure")
                .about("Configure settings for specific UART/USB port")
                .arg(
                    Arg::new("port")
                        .long("select")
                        .required(true)
                        .default_value("usb")
                        .value_parser(value_parser!(String))
                        .long_help(
                            "Apply specific configuration to the selected port. Supported: usb, uart1, uart2.
Configuration includes: protocol in/out, data-bits, stop-bits, parity, baud-rate",
                        ),
                    )
                .arg(
                    Arg::new("cfg-baud")
                        .value_name("baud")
                        .long("baud")
                        .required(false)
                        .default_value("9600")
                        .value_parser(value_parser!(u32))
                        .help("Baud rate to set"),
                )
                .arg(
                    Arg::new("stop-bits")
                        .long("stop-bits")
                        .help("Number of stop bits to set")
                        .required(false)
                        .value_parser(["1", "2"])
                        .default_value("1"),
                )
                .arg(
                    Arg::new("data-bits")
                        .long("data-bits")
                        .help("Number of data bits to set")
                        .required(false)
                        .value_parser(["7", "8"])
                        .default_value("8"),
                )
                .arg(
                    Arg::new("parity")
                        .long("parity")
                        .help("Parity to set")
                        .required(false)
                        .value_parser(["even", "odd"]),
                )
                .arg(
                    Arg::new("in-ublox")
                        .long("in-ublox")
                        .default_value("true")
                        .action(clap::ArgAction::SetTrue)
                        .help("Toggle receiving UBX proprietary protocol on port"),
                )
                .arg(
                    Arg::new("in-nmea")
                        .long("in-nmea")
                        .default_value("false")
                        .action(clap::ArgAction::SetTrue)
                        .help("Toggle receiving NMEA protocol on port"),
                )
                .arg(
                    Arg::new("in-rtcm")
                        .long("in-rtcm")
                        .default_value("false")
                        .action(clap::ArgAction::SetTrue)
                        .help("Toggle receiving RTCM protocol on port"),
                )
                .arg(
                    Arg::new("in-rtcm3")
                        .long("in-rtcm3")
                        .default_value("false")
                        .action(clap::ArgAction::SetTrue)
                        .help(
                            "Toggle receiving RTCM3 protocol on port.
        Not supported on uBlox protocol versions below 20",
                        ),
                )
                .arg(
                    Arg::new("out-ublox")
                        .long("out-ublox")
                        .action(clap::ArgAction::SetTrue)
                        .help("Toggle sending UBX proprietary protocol on port"),
                )
                .arg(
                    Arg::new("out-nmea")
                        .long("out-nmea")
                        .action(clap::ArgAction::SetTrue)
                        .help("Toggle sending NMEA protocol on port"),
                )
                .arg(
                    Arg::new("out-rtcm3")
                        .long("out-rtcm3")
                        .action(clap::ArgAction::SetTrue)
                        .help(
                            "Toggle seding RTCM3 protocol on port.
        Not supported on uBlox protocol versions below 20",
                        ),
                ),
        )
        .get_matches()
}
