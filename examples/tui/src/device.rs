use clap::ArgMatches;
use serialport::{
    DataBits as SerialDataBits, FlowControl as SerialFlowControl, Parity as SerialParity,
    StopBits as SerialStopBits,
};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;
use tracing::{debug, error, info, trace, warn};
use ublox::*;

use crate::app::{
    EsfAlgImuAlignmentWidgetState, EsfAlgStatusWidgetState, EsfSensorWidget, EsfSensorsWidgetState,
    MonVersionWidgetState, NavPvtWidgetState, UbxStatus,
};

pub struct Device {
    port: Box<dyn serialport::SerialPort>,
    parser: Parser<Vec<u8>>,
}

impl Device {
    pub fn new(port: Box<dyn serialport::SerialPort>) -> Device {
        let parser = Parser::default();
        Device { port, parser }
    }

    pub fn build(cli: &ArgMatches) -> Device {
        let port = cli
            .get_one::<String>("port")
            .expect("Expected required 'port' cli argumnet");

        let baud = cli.get_one::<u32>("baud").cloned().unwrap_or(9600);
        let stop_bits = match cli.get_one::<String>("stop-bits").map(|s| s.as_str()) {
            Some("2") => SerialStopBits::Two,
            _ => SerialStopBits::One,
        };
        let data_bits = match cli.get_one::<String>("data-bits").map(|s| s.as_str()) {
            Some("7") => SerialDataBits::Seven,
            Some("8") => SerialDataBits::Eight,
            _ => {
                error!("Number of DataBits supported by uBlox is either 7 or 8");
                std::process::exit(1);
            },
        };

        let parity = match cli.get_one::<String>("parity").map(|s| s.as_str()) {
            Some("odd") => SerialParity::Even,
            Some("even") => SerialParity::Odd,
            _ => SerialParity::None,
        };

        let serialport_builder = serialport::new(port, baud)
            .stop_bits(stop_bits)
            .data_bits(data_bits)
            .timeout(Duration::from_millis(10))
            .parity(parity)
            .flow_control(SerialFlowControl::None);

        debug!("{:?}", &serialport_builder);
        let port = serialport_builder.open().unwrap_or_else(|e| {
            error!("Failed to open \"{}\". Error: {}", port, e);
            ::std::process::exit(1);
        });

        let mut device = Device::new(port);
        device.configure_uart_ports(cli);
        device.configure_ubx_msgs(cli);
        device
    }

    fn configure_uart_ports(&mut self, cli: &ArgMatches) {
        // Parse cli for configuring specific uBlox UART port
        if let Some(("configure", sub_matches)) = cli.subcommand() {
            let (port_id, port_name) =
                match sub_matches.get_one::<String>("port").map(|s| s.as_str()) {
                    Some(x) if x == "usb" => (Some(UartPortId::Usb), x),
                    Some(x) if x == "uart1" => (Some(UartPortId::Uart1), x),
                    Some(x) if x == "uart2" => (Some(UartPortId::Uart2), x),
                    _ => (None, ""),
                };

            let baud = sub_matches.get_one::<u32>("baud").cloned().unwrap_or(9600);

            let stop_bits = match sub_matches
                .get_one::<String>("stop-bits")
                .map(|s| s.as_str())
            {
                Some("2") => SerialStopBits::Two,
                _ => SerialStopBits::One,
            };

            let data_bits = match sub_matches
                .get_one::<String>("data-bits")
                .map(|s| s.as_str())
            {
                Some("7") => SerialDataBits::Seven,
                Some("8") => SerialDataBits::Eight,
                _ => {
                    error!("Number of DataBits supported by uBlox is either 7 or 8");
                    std::process::exit(1);
                },
            };

            let parity = match sub_matches.get_one::<String>("parity").map(|s| s.as_str()) {
                Some("odd") => SerialParity::Even,
                Some("even") => SerialParity::Odd,
                _ => SerialParity::None,
            };
            let inproto = match (
                sub_matches.get_flag("in-ublox"),
                sub_matches.get_flag("in-nmea"),
                sub_matches.get_flag("in-rtcm"),
                sub_matches.get_flag("in-rtcm3"),
            ) {
                (true, false, false, false) => InProtoMask::UBLOX,
                (false, true, false, false) => InProtoMask::NMEA,
                (false, false, true, false) => InProtoMask::RTCM,
                (false, false, false, true) => InProtoMask::RTCM3,
                (true, true, false, false) => {
                    InProtoMask::union(InProtoMask::UBLOX, InProtoMask::NMEA)
                },
                (true, false, true, false) => {
                    InProtoMask::union(InProtoMask::UBLOX, InProtoMask::RTCM)
                },
                (true, false, false, true) => {
                    InProtoMask::union(InProtoMask::UBLOX, InProtoMask::RTCM3)
                },
                (false, true, true, false) => {
                    InProtoMask::union(InProtoMask::NMEA, InProtoMask::RTCM)
                },
                (false, true, false, true) => {
                    InProtoMask::union(InProtoMask::NMEA, InProtoMask::RTCM3)
                },
                (true, true, true, false) => InProtoMask::union(
                    InProtoMask::union(InProtoMask::UBLOX, InProtoMask::NMEA),
                    InProtoMask::RTCM,
                ),
                (true, true, false, true) => InProtoMask::union(
                    InProtoMask::union(InProtoMask::UBLOX, InProtoMask::NMEA),
                    InProtoMask::RTCM3,
                ),
                (_, _, true, true) => {
                    error!("Cannot use RTCM and RTCM3 simultaneously. Choose one or the other");
                    std::process::exit(1)
                },
                (false, false, false, false) => InProtoMask::UBLOX,
            };

            let outproto = match (
                sub_matches.get_flag("out-ublox"),
                sub_matches.get_flag("out-nmea"),
                sub_matches.get_flag("out-rtcm3"),
            ) {
                (true, false, false) => OutProtoMask::UBLOX,
                (false, true, false) => OutProtoMask::NMEA,
                (false, false, true) => OutProtoMask::RTCM3,
                (true, true, false) => OutProtoMask::union(OutProtoMask::UBLOX, OutProtoMask::NMEA),
                (true, false, true) => {
                    OutProtoMask::union(OutProtoMask::UBLOX, OutProtoMask::RTCM3)
                },
                (false, true, true) => OutProtoMask::union(OutProtoMask::NMEA, OutProtoMask::RTCM3),
                (true, true, true) => OutProtoMask::union(
                    OutProtoMask::union(OutProtoMask::UBLOX, OutProtoMask::NMEA),
                    OutProtoMask::RTCM3,
                ),
                (false, false, false) => OutProtoMask::UBLOX,
            };

            if let Some(port_id) = port_id {
                info!("Configuring '{}' port ...", port_name.to_uppercase());
                self.write_all(
                    &CfgPrtUartBuilder {
                        portid: port_id,
                        reserved0: 0,
                        tx_ready: 0,
                        mode: UartMode::new(
                            ublox_databits(data_bits),
                            ublox_parity(parity),
                            ublox_stopbits(stop_bits),
                        ),
                        baud_rate: baud,
                        in_proto_mask: inproto,
                        out_proto_mask: outproto,
                        flags: 0,
                        reserved5: 0,
                    }
                    .into_packet_bytes(),
                )
                .expect("Could not configure UBX-CFG-PRT-UART");
                self.wait_for_ack::<CfgPrtUart>()
                    .expect("Could not acknowledge UBX-CFG-PRT-UART msg");
            }
        }
    }

    fn configure_ubx_msgs(&mut self, _cli: &ArgMatches) {
        // Enable the NavPvt packet
        // By setting 1 in the array below, we enable the NavPvt message for Uart1, Uart2 and USB
        // The other positions are for I2C, SPI, etc. Consult your device manual.
        info!("Enable UBX-NAV-PVT message on all serial ports: USB, UART1 and UART2 ...");
        self.write_all(
            &CfgMsgAllPortsBuilder::set_rate_for::<NavPvt>([0, 1, 1, 1, 0, 0]).into_packet_bytes(),
        )
        .expect("Could not configure ports for UBX-NAV-PVT");

        self.wait_for_ack::<CfgMsgAllPorts>()
            .expect("Could not acknowledge UBX-CFG-PRT-UART msg");

        // Send a packet request for the MonVer packet
        self.write_all(&UbxPacketRequest::request_for::<MonVer>().into_packet_bytes())
            .expect("Unable to write request/poll for UBX-MON-VER message");

        self.write_all(&UbxPacketRequest::request_for::<EsfAlg>().into_packet_bytes())
            .expect("Unable to write request/poll for UBX-ESF-ALG message");

        self.write_all(&UbxPacketRequest::request_for::<EsfStatus>().into_packet_bytes())
            .expect("Unable to write request/poll for UBX-ESF-STATUS message");
    }

    pub fn run(mut self, sender: Sender<UbxStatus>) {
        info!("Opened uBlox device, waiting for messages...");
        thread::spawn(move || loop {
            let res = self.update(|packet| match packet {
                PacketRef::MonVer(pkg) => {
                    trace!("{:?}", pkg);
                    info!(
                        "SW version: {} HW version: {}; Extensions: {:?}",
                        pkg.software_version(),
                        pkg.hardware_version(),
                        pkg.extension().collect::<Vec<&str>>()
                    );
                    let mut state = MonVersionWidgetState::default();

                    state
                        .software_version
                        .copy_from_slice(pkg.software_version_raw());
                    state
                        .hardware_version
                        .copy_from_slice(pkg.hardware_version_raw());

                    for s in pkg.extension() {
                        state.extensions.push_str(s);
                    }

                    sender.send(UbxStatus::MonVer(Box::new(state))).unwrap();
                },
                PacketRef::NavPvt(pkg) => {
                    let mut state = NavPvtWidgetState {
                        time_tag: (pkg.itow() / 1000) as f64,
                        ..Default::default()
                    };

                    state.flags2 = pkg.flags2();

                    if pkg.flags2().contains(NavPvtFlags2::CONFIRMED_AVAI) {
                        state.day = pkg.day();
                        state.month = pkg.month();
                        state.year = pkg.year();
                        state.hour = pkg.hour();
                        state.min = pkg.min();
                        state.sec = pkg.sec();
                        state.nanosecond = pkg.nanosec();

                        state.utc_time_accuracy = pkg.time_accuracy();
                    }

                    state.position_fix_type = pkg.fix_type();
                    state.fix_flags = pkg.flags();

                    state.lat = pkg.latitude();
                    state.lon = pkg.longitude();
                    state.height = pkg.height_above_ellipsoid();
                    state.msl = pkg.height_msl();

                    state.vel_ned = (pkg.vel_north(), pkg.vel_east(), pkg.vel_down());

                    state.speed_over_ground = pkg.ground_speed_2d();
                    state.heading_motion = pkg.heading_motion();
                    state.heading_vehicle = pkg.heading_vehicle();

                    state.magnetic_declination = pkg.magnetic_declination();

                    state.pdop = pkg.pdop();

                    state.satellites_used = pkg.num_satellites();

                    state.invalid_llh = pkg.flags3().invalid_llh();
                    state.position_accuracy = (pkg.horizontal_accuracy(), pkg.vertical_accuracy());
                    state.velocity_accuracy = pkg.speed_accuracy();
                    state.heading_accuracy = pkg.heading_accuracy();
                    state.magnetic_declination_accuracy = pkg.magnetic_declination_accuracy();

                    sender.send(UbxStatus::Pvt(Box::new(state))).unwrap();
                    debug!("{:?}", pkg);
                },
                PacketRef::EsfAlg(pkg) => {
                    let mut state = EsfAlgImuAlignmentWidgetState {
                        time_tag: (pkg.itow() / 1000) as f64,
                        ..Default::default()
                    };
                    state.roll = pkg.roll();
                    state.pitch = pkg.pitch();
                    state.yaw = pkg.yaw();

                    state.auto_alignment = pkg.flags().auto_imu_mount_alg_on();
                    state.alignment_status = pkg.flags().status();

                    if pkg.error().contains(EsfAlgError::ANGLE_ERROR) {
                        state.angle_singularity = true;
                    }

                    sender.send(UbxStatus::EsfAlgImu(state)).unwrap();
                    // debug!("{:?}", pkg);
                },

                PacketRef::EsfStatus(pkg) => {
                    let mut alg_state = EsfAlgStatusWidgetState {
                        time_tag: (pkg.itow() / 1000) as f64,
                        ..Default::default()
                    };
                    alg_state.fusion_mode = pkg.fusion_mode();

                    alg_state.imu_status = pkg.init_status2().imu_init_status();
                    alg_state.ins_status = pkg.init_status1().ins_initialization_status();
                    alg_state.ins_status = pkg.init_status1().ins_initialization_status();
                    alg_state.wheel_tick_sensor_status =
                        pkg.init_status1().wheel_tick_init_status();

                    let mut sensors = EsfSensorsWidgetState::default();
                    let mut sensor_state = EsfSensorWidget::default();
                    for s in pkg.data() {
                        if s.sensor_used() {
                            sensor_state.sensor_type = s.sensor_type();
                            sensor_state.freq = s.freq();
                            sensor_state.faults = s.faults();
                            sensor_state.calib_status = s.calibration_status();
                            sensor_state.time_status = s.time_status();
                            sensors.sensors.push(sensor_state.clone());
                        }
                    }

                    sender.send(UbxStatus::EsfAlgStatus(alg_state)).unwrap();
                    sender.send(UbxStatus::EsfAlgSensors(sensors)).unwrap();
                    // debug!("{:?}", pkg);
                },

                _ => {
                    trace!("{:?}", packet);
                },
            });
            if let Err(e) = res {
                error!("Stopping UBX messages parsing thread. Failed to parse incoming UBX packet: {e}");
            }
        });
    }
    pub fn write_all(&mut self, data: &[u8]) -> std::io::Result<()> {
        self.port.write_all(data)
    }

    pub fn update<T: FnMut(PacketRef)>(&mut self, mut cb: T) -> std::io::Result<()> {
        loop {
            const MAX_PAYLOAD_LEN: usize = 1240;
            let mut local_buf = [0; MAX_PAYLOAD_LEN];
            let nbytes = self.read_port(&mut local_buf)?;
            if nbytes == 0 {
                break;
            }

            // Parser.consume adds the buffer to its internal buffer, and
            // returns an iterator-like object we can use to process the packets
            let mut it = self.parser.consume(&local_buf[..nbytes]);
            loop {
                match it.next() {
                    Some(Ok(packet)) => {
                        cb(packet);
                    },
                    Some(Err(e)) => {
                        trace!("Received malformed packet, ignoring it. Error: {e}");
                    },
                    None => {
                        // We've eaten all the packets we have
                        break;
                    },
                }
            }
        }
        Ok(())
    }

    pub fn wait_for_ack<T: UbxPacketMeta>(&mut self) -> std::io::Result<()> {
        let mut found_packet = false;
        let start = std::time::SystemTime::now();
        let timeout = Duration::from_secs(3);
        while !found_packet {
            self.update(|packet| {
                if let PacketRef::AckAck(ack) = packet {
                    if ack.class() == T::CLASS && ack.msg_id() == T::ID {
                        found_packet = true;
                    }
                }
            })?;

            if start.elapsed().unwrap().as_millis() > timeout.as_millis() {
                error!("Did not receive ACK message for request");
                break;
            }
        }
        Ok(())
    }

    /// Reads the serial port, converting timeouts into "no data received"
    fn read_port(&mut self, output: &mut [u8]) -> std::io::Result<usize> {
        match self.port.read(output) {
            Ok(b) => Ok(b),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::TimedOut {
                    Ok(0)
                } else {
                    Err(e)
                }
            },
        }
    }
}

fn ublox_stopbits(s: SerialStopBits) -> StopBits {
    // Seriaport crate doesn't support the other StopBits option of uBlox
    match s {
        SerialStopBits::One => StopBits::One,
        SerialStopBits::Two => StopBits::Two,
    }
}

fn ublox_databits(d: SerialDataBits) -> DataBits {
    match d {
        SerialDataBits::Seven => DataBits::Seven,
        SerialDataBits::Eight => DataBits::Eight,
        _ => {
            warn!("uBlox only supports Seven or Eight data bits. Setting to DataBits to 8");
            DataBits::Eight
        },
    }
}

fn ublox_parity(v: SerialParity) -> Parity {
    match v {
        SerialParity::Even => Parity::Even,
        SerialParity::Odd => Parity::Odd,
        SerialParity::None => Parity::None,
    }
}
