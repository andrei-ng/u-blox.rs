use std::{io::ErrorKind, process::exit};

use ublox::*;

fn main() {
    let mut cli = ublox_device::cli::CommandBuilder::default().build();
    cli = cli
        .about("Demonstrate usage of uBlox package for ESF (Extended Sensor Fusion) mode e.g., when used for ADR")
        .name("basic_cli")
        .author(clap::crate_authors!());

    let serialport = ublox_device::cli::Command::serialport(cli.clone());

    let mut device = ublox_device::Device::new(serialport);

    // Configure the device to talk UBX
    println!("Configuring UART1 port ...");
    device
        .write_all(
            &CfgPrtUartBuilder {
                portid: UartPortId::Uart1,
                reserved0: 0,
                tx_ready: 0,
                mode: UartMode::new(DataBits::Eight, Parity::None, StopBits::One),
                baud_rate: ublox_device::cli::Command::arg_boud(cli),
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

    #[cfg(feature = "ubx_proto23")]
    device
        .write_all(
            &CfgMsgAllPortsBuilder::set_rate_for::<HnrAtt>([0, 1, 0, 1, 0, 0]).into_packet_bytes(),
        )
        .expect("Could not configure ports for UBX-HNR-ATT");

    #[cfg(feature = "ubx_proto23")]
    device
        .write_all(
            &CfgMsgAllPortsBuilder::set_rate_for::<HnrIns>([0, 1, 0, 1, 0, 0]).into_packet_bytes(),
        )
        .expect("Could not configure ports for UBX-HNR-INS");

    #[cfg(feature = "ubx_proto23")]
    device
        .write_all(
            &CfgMsgAllPortsBuilder::set_rate_for::<HnrPvt>([0, 1, 0, 1, 0, 0]).into_packet_bytes(),
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

    // Configure Auto Alignment-ON for the IMU
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

    let mut handler = PkgHandler;
    loop {
        if let Err(e) = device.process(&mut handler) {
            match e.kind() {
                ErrorKind::Interrupted => {
                    eprintln!("Received signal interrupt. Exiting ...");
                    exit(1)
                },
                ErrorKind::BrokenPipe => {
                    eprintln!("Broken Pipe. Exiting ...");
                    exit(1)
                },
                _ => {
                    println!("Failed to parse UBX packets due to: {e}");
                },
            }
        }
    }
}

struct PkgHandler;

impl ublox_device::UbxPacketHandler for PkgHandler {
    fn handle(&mut self, packet: ublox::PacketRef) {
        match packet {
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
        }
    }
}
