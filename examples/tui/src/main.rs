use std::{error::Error, sync::mpsc::channel};

use clap::ArgMatches;
use device::Device;

mod app;
mod cli;
mod device;
mod logging;
mod tui;
mod ui;

fn main() -> Result<(), Box<dyn Error>> {
    let cli = cli::parse_args();

    if cli.get_flag("debug-mode") {
        debug_mode(&cli);
    } else {
        crate::tui::run(&cli)?;
    }
    Ok(())
}

fn debug_mode(cli: &ArgMatches) {
    use log::error;
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .parse_env("TUI_LOGLEVEL")
        .init();

    let (ubx_msg_tx, ubx_msg_rs) = channel();

    let device = Device::build(cli);
    device.run(ubx_msg_tx);

    loop {
        match ubx_msg_rs.recv() {
            Ok(_) => {
                // We don't do anything with the received messages as data as this is intended for the TUI Widgets;
            },
            Err(e) => error!("Error: {e}"),
        }
    }
}
