use clap::{value_parser, Arg, Command};
use serialport::{
    DataBits as SerialDataBits, FlowControl as SerialFlowControl, Parity as SerialParity,
    StopBits as SerialStopBits,
};
use std::time::Duration;
use ublox::*;

fn main() {
    let matches = Command::new("uBlox CLI example program")
        .author(clap::crate_authors!())
        .about("Demonstrates usage of the Rust uBlox API")
        .arg_required_else_help(true)
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
                .value_parser(value_parser!(u32))
                .help("Baud rate of the port"),
        )
        .arg(
            Arg::new("stop-bits")
                .long("stop-bits")
                .help("Number of stop bits to use")
                .required(false)
                .value_parser(["1", "2"])
                .default_value("1"),
        )
        .arg(
            Arg::new("data-bits")
                .long("data-bits")
                .help("Number of data bits to use")
                .required(false)
                .value_parser(["5", "6", "7", "8"])
                .default_value("8"),
        )
        .get_matches();

    let port = matches
        .get_one::<String>("port")
        .expect("Expected required 'port' cli argumnet");
    let baud = matches.get_one::<u32>("baud").cloned().unwrap_or(115200);
    let stop_bits = match matches.get_one::<String>("stop-bits").map(|s| s.as_str()) {
        Some("2") => SerialStopBits::Two,
        _ => SerialStopBits::One,
    };
    let data_bits = match matches.get_one::<String>("data-bits").map(|s| s.as_str()) {
        Some("5") => SerialDataBits::Five,
        Some("6") => SerialDataBits::Six,
        Some("7") => SerialDataBits::Seven,
        _ => SerialDataBits::Eight,
    };

    let builder = serialport::new(port, baud)
        .stop_bits(stop_bits)
        .data_bits(data_bits)
        .timeout(Duration::from_millis(1))
        .parity(SerialParity::None)
        .flow_control(SerialFlowControl::None);

    println!("{:?}", &builder);
    let port = builder.open().unwrap_or_else(|e| {
        eprintln!("Failed to open \"{}\". Error: {}", port, e);
        ::std::process::exit(1);
    });
    let mut device = Device::new(port);

    // Configure the device to talk UBX
    println!("Configuring UART1 port ...");
    device
        .write_all(
            &CfgPrtUartBuilder {
                portid: UartPortId::Uart1,
                reserved0: 0,
                tx_ready: 0,
                mode: UartMode::new(DataBits::Eight, Parity::None, StopBits::One),
                baud_rate: baud,
                in_proto_mask: InProtoMask::UBLOX,
                out_proto_mask: OutProtoMask::union(OutProtoMask::NMEA, OutProtoMask::UBLOX),
                flags: 0,
                reserved5: 0,
            }
            .into_packet_bytes(),
        )
        .expect("Could not configure UBX-CFG-PRT-UART");
    device
        .wait_for_ack::<CfgPrtUart>()
        .expect("Could not acknowledge UBX-CFG-PRT-UART msg");

    device
        .write_all(
            &CfgMsgAllPortsBuilder::set_rate_for::<HnrAtt>([0, 0, 0, 0, 0, 0]).into_packet_bytes(),
        )
        .expect("Could not configure ports for UBX-HNR-ATT");

    device
        .write_all(
            &CfgMsgAllPortsBuilder::set_rate_for::<HnrIns>([0, 0, 0, 0, 0, 0]).into_packet_bytes(),
        )
        .expect("Could not configure ports for UBX-HNR-INS");

    device
        .write_all(
            &CfgMsgAllPortsBuilder::set_rate_for::<HnrPvt>([0, 0, 0, 0, 0, 0]).into_packet_bytes(),
        )
        .expect("Could not configure ports for UBX-HNR-PVT");

    device
        .write_all(
            &CfgMsgAllPortsBuilder::set_rate_for::<EsfIns>([0, 1, 0, 1, 0, 0]).into_packet_bytes(),
        )
        .expect("Could not configure ports for UBX-ESF-INS");

    device
        .write_all(
            &CfgMsgAllPortsBuilder::set_rate_for::<EsfAlg>([0, 1, 0, 1, 0, 0]).into_packet_bytes(),
        )
        .expect("Could not configure ports for UBX-ESF-ALG");

    device
        .write_all(
            &CfgMsgAllPortsBuilder::set_rate_for::<EsfStatus>([0, 1, 0, 1, 0, 0])
                .into_packet_bytes(),
        )
        .expect("Could not configure ports for UBX-ESF-STATUS");

    device
        .write_all(
            &CfgMsgAllPortsBuilder::set_rate_for::<EsfMeas>([0, 1, 0, 1, 0, 0]).into_packet_bytes(),
        )
        .expect("Could not configure ports for UBX-ESF-MEAS");

    device
        .write_all(
            &CfgMsgAllPortsBuilder::set_rate_for::<NavPvt>([0, 0, 0, 0, 0, 0]).into_packet_bytes(),
        )
        .expect("Could not configure ports for UBX-NAV-PVT");

    // Send a packet request for the MonVer packet
    device
        .write_all(&UbxPacketRequest::request_for::<MonVer>().into_packet_bytes())
        .expect("Unable to write request/poll for UBX-MON-VER message");

    // Configure Auto Alignment on for IMU
    let mut flags = CfgEsfAlgFlags::default();
    flags.set_auto_imu_mount_alg(true);
    device
        .write_all(
            &ublox::CfgEsfAlgBuilder {
                flags,
                yaw: 5.0,
                roll: 1.0,
                pitch: 2.0,
            }
            .into_packet_bytes(),
        )
        .expect("Could not write UBX-CFG-ESFALG msg due to: {e}");

    device
        .write_all(&UbxPacketRequest::request_for::<CfgEsfAlg>().into_packet_bytes())
        .expect("Unable to write request/poll for UBX-CFG-ESFALG message");

    device
        .write_all(
            &CfgMsgAllPortsBuilder::set_rate_for::<CfgEsfAlg>([0, 1, 0, 1, 0, 0])
                .into_packet_bytes(),
        )
        .expect("Could not configure ports for UBX-CFG-ESF-ALG");

    // Configure Wheel Speed for ESF
    device
        .write_all(
            &ublox::CfgEsfWtBuilder {
                flags1: CfgEsfWtFlags1::USE_WHEEL_TICK_SPEED,
                wt_frequency: 13,
                ..Default::default()
            }
            .into_packet_bytes(),
        )
        .expect("Could not write UBX-CFG-ESFALG msg due to: {e}");

    device
        .write_all(&UbxPacketRequest::request_for::<CfgEsfWt>().into_packet_bytes())
        .expect("Unable to write request/poll for UBX-CFG-ESF-WT message");

    device
        .write_all(
            &CfgMsgAllPortsBuilder::set_rate_for::<CfgEsfWt>([0, 1, 0, 1, 0, 0])
                .into_packet_bytes(),
        )
        .expect("Could not configure ports for UBX-CFG-ESF-WT");

    println!("Opened uBlox device, waiting for messages...");
    loop {
        device
            .update(|packet| match packet {
                PacketRef::MonVer(packet) => {
                    println!(
                        "SW version: {} HW version: {}; Extensions: {:?}",
                        packet.software_version(),
                        packet.hardware_version(),
                        packet.extension().collect::<Vec<&str>>()
                    );
                },
                PacketRef::CfgEsfWt(msg) => {
                    println!("Received: {:?}", msg);
                },
                PacketRef::CfgEsfAlg(msg) => {
                    println!("Received: {:?}", msg);
                },


                PacketRef::EsfStatus(status) => {
                    println!(
                        "EsfStatus: tow: {}, version: {}, {:?},{:?}, fusion_mode: {:?}, num_sens: {}",
                        status.itow(),
                        status.version(),
                        status.init_status1(),
                        status.init_status2(),
                        status.fusion_mode(),
                        status.num_sens(),
                    );
                    for s in status.data() {
                        println!("{:?}", s);
                    }
                },

                PacketRef::EsfMeas(msg) => {
                    println!("{:?}", msg);
                    println!("time_tag: {}", msg.time_tag());
                    for s in msg.data() {
                        println!("{:?}", s);
                        println!("{:?}", s.value());
                    }
                    println!("calib_tag: {:?}", msg.calib_tag());
                },
                _ => {
                    {} //println!("{:?}", packet);
                },
            })
            .unwrap();
    }
}

struct Device {
    port: Box<dyn serialport::SerialPort>,
    parser: Parser<Vec<u8>>,
}

impl Device {
    pub fn new(port: Box<dyn serialport::SerialPort>) -> Device {
        let parser = Parser::default();
        Device { port, parser }
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

            // parser.consume adds the buffer to its internal buffer, and
            // returns an iterator-like object we can use to process the packets
            let mut it = self.parser.consume(&local_buf[..nbytes]);
            loop {
                match it.next() {
                    Some(Ok(packet)) => {
                        cb(packet);
                    },
                    Some(Err(_)) => {
                        // Received a malformed packet, ignore it
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
                eprintln!("Did not receive ACK message for request");
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
