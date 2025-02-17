use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::{Line, Span},
    widgets::{
        canvas::{Canvas, Circle, Map, MapResolution},
        Block, Cell, Paragraph, Row, Table, Tabs, Widget, Wrap,
    },
    Frame,
};

use tui_logger::{TuiLoggerLevelOutput, TuiLoggerWidget};
use ublox::{
    EsfAlgStatus, EsfSensorFaults, EsfSensorStatusCalibration, EsfSensorStatusTime, EsfSensorType,
    EsfStatusFusionMode, EsfStatusImuInit, EsfStatusInsInit, EsfStatusMountAngle,
    EsfStatusWheelTickInit, GpsFix, NavPvtFlags, NavPvtFlags2,
};

use crate::app::App;

#[derive(Debug, Default)]
pub struct LogWidget;

impl Widget for &mut LogWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        TuiLoggerWidget::default()
            .block(Block::bordered().title("Log"))
            .style_error(Style::default().fg(Color::Red))
            .style_warn(Style::default().fg(Color::Yellow))
            .style_info(Style::default().fg(Color::Green))
            .style_debug(Style::default().fg(Color::White))
            .style_trace(Style::default().fg(Color::Magenta))
            .output_separator(':')
            .output_timestamp(Some("%F %H:%M:%S%.3f".to_string()))
            .output_level(Some(TuiLoggerLevelOutput::Long))
            .output_target(false)
            .output_file(false)
            .output_line(false)
            .style(Style::default().fg(ratatui::style::Color::White))
            .render(area, buf);

        // TuiLoggerSmartWidget::default()
        //     .title_log("Log")
        //     .style_error(Style::default().fg(Color::Red))
        //     .style_debug(Style::default().fg(Color::Green))
        //     .style_warn(Style::default().fg(Color::Yellow))
        //     .style_trace(Style::default().fg(Color::Magenta))
        //     .style_info(Style::default().fg(Color::Cyan))
        //     .output_separator(':')
        //     .output_timestamp(Some("%H:%M:%S".to_string()))
        //     .output_level(Some(TuiLoggerLevelOutput::Abbreviated))
        //     .output_target(true)
        //     .output_file(true)
        //     .output_line(true)
        // .state(self.selected_state())
        // .render(area, buf);
    }
}

pub fn draw(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(frame.area());
    let tabs = app
        .tabs
        .titles
        .iter()
        .map(|t| Line::from(Span::styled(*t, Style::default().fg(Color::Green))))
        .collect::<Tabs>()
        .block(Block::bordered().title(app.title))
        .highlight_style(Style::default().fg(Color::Yellow))
        .select(app.tabs.index);
    frame.render_widget(tabs, chunks[0]);
    match app.tabs.index {
        0 => draw_state_tab(frame, app, chunks[1]),
        1 => draw_version_info(frame, app, chunks[1]),
        2 => draw_map(frame, app, chunks[1]),
        _ => {},
    };
}

fn draw_state_tab(frame: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::vertical([Constraint::Length(24), Constraint::Min(7)]).split(area);
    render_pvt_and_esf_statuses(frame, chunks[0], app);
    frame.render_widget(&mut app.log_widget, chunks[1]);
}

fn draw_version_info(frame: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::vertical([Constraint::Length(24), Constraint::Min(7)]).split(area);
    render_monver(frame, chunks[0], app);
    frame.render_widget(&mut app.log_widget, chunks[1]);
}

fn render_pvt_and_esf_statuses(frame: &mut Frame, area: Rect, app: &mut App) {
    let chunks =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(area);

    render_pvt_state(frame, chunks[0], app);
    render_esf_status(frame, chunks[1], app);
}

fn render_pvt_state(frame: &mut Frame, area: Rect, app: &mut App) {
    let time_tag = format!("{:.3}", app.pvt_state.time_tag);
    let position = format!(
        "{:.4}, {:.4}, {:.4}, {:.4}",
        app.pvt_state.lat, app.pvt_state.lon, app.pvt_state.height, app.pvt_state.msl
    );
    let time_accuracy = format!("{}", app.pvt_state.utc_time_accuracy);
    let position_accuracy = format!(
        "{:.2}, {:.2}",
        app.pvt_state.position_accuracy.0, app.pvt_state.position_accuracy.1
    );

    let velocity_ned = format!(
        "{:.3}, {:.3}, {:.3}",
        app.pvt_state.vel_ned.0, app.pvt_state.vel_ned.1, app.pvt_state.vel_ned.2
    );

    let velocity_heading_acc = format!(
        "{:.3}, {:.3}",
        app.pvt_state.velocity_accuracy, app.pvt_state.heading_accuracy
    );

    let heading_info = format!(
        "{:.3}, {:.3}",
        app.pvt_state.heading_motion, app.pvt_state.heading_vehicle
    );

    let magnetic_declination = format!(
        "{:.2}, {:.2}",
        app.pvt_state.magnetic_declination, app.pvt_state.magnetic_declination_accuracy
    );

    let gps_fix = match app.pvt_state.position_fix_type {
        GpsFix::DeadReckoningOnly => "DR",
        GpsFix::Fix2D => "2D Fix",
        GpsFix::Fix3D => "3D Fix",
        GpsFix::GPSPlusDeadReckoning => "3D + DR",
        GpsFix::TimeOnlyFix => "Time Only",
        _ => "No Fix",
    };

    let mut fix_flags = String::default();
    if app.pvt_state.fix_flags.contains(NavPvtFlags::GPS_FIX_OK) {
        fix_flags = "FixOK".to_string();
    }
    if app.pvt_state.fix_flags.contains(NavPvtFlags::DIFF_SOLN) {
        fix_flags.push_str(" + DGNSS");
    }

    let utc_date_time = format!(
        "{:02}-{:02}-{} {:02}:{:02}:{:02} {:09}",
        app.pvt_state.day,
        app.pvt_state.month,
        app.pvt_state.year,
        app.pvt_state.hour,
        app.pvt_state.min,
        app.pvt_state.sec,
        app.pvt_state.nanosecond,
    );

    let mut time_date_confirmation = if app.pvt_state.flags2.contains(NavPvtFlags2::CONFIRMED_DATE)
    {
        "Date: CONFIRMED".to_string()
    } else {
        "Date: ?".to_string()
    };

    if app.pvt_state.flags2.contains(NavPvtFlags2::CONFIRMED_TIME) {
        time_date_confirmation.push_str(", Time: CONFIRMED");
    } else {
        time_date_confirmation.push_str(", Time: ?");
    }
    let rows = [
        Row::new(["GPS Time Tag", &time_tag, "[s]"]),
        Row::new(["UTC Date Time", &utc_date_time, ""]),
        Row::new(["UTC Date Time Confirmation", &time_date_confirmation, ""]),
        Row::new(["UTC Time Accuracy", &time_accuracy, "[ns]"]),
        Row::new(["Position Fix Type", gps_fix, ""]),
        Row::new(["Fix Flags", &fix_flags, ""]),
        Row::new(["PSM State", "n/a", ""]),
        Row::new(["Lat,Lon,Height,MSL", &position, "[deg,deg,m,m]"]),
        Row::new([
            "Invalid Position",
            if app.pvt_state.invalid_llh {
                "Yes"
            } else {
                "No"
            },
            "",
        ]),
        Row::new(["Position Accuracy Horiz, Vert", &position_accuracy, "[m,m]"]),
        Row::new(["Velocity NED", &velocity_ned, "[m/s,m/s,m/s]"]),
        Row::new([
            "Velocity, Heading Accuracy",
            &velocity_heading_acc,
            "[m/s, deg]",
        ]),
        Row::new([
            Cell::from("Speed over Ground"),
            Cell::from(format!("{:.4}", app.pvt_state.speed_over_ground)),
            Cell::from("[m/s]"),
        ]),
        Row::new([
            "Heading Motion, Heading Vehicle",
            &heading_info,
            "[deg,deg]",
        ]),
        Row::new([
            "Magnetic Declination, Declination Accuracy",
            &magnetic_declination,
            "[deg,deg]",
        ]),
        Row::new([
            Cell::from("PDOP"),
            Cell::from(format!("{:.3}", app.pvt_state.pdop)),
            Cell::from(""),
        ]),
        Row::new([
            Cell::from("#SVs Used"),
            Cell::from(app.pvt_state.satellites_used.to_string()),
            Cell::from(""),
        ]),
        Row::new(["Carrier Range Status", "Not Used", ""]),
        Row::new(["Age of recent differential correction", "???", "[sec]"]),
        Row::new(["NMA Fix Status", "???", ""]),
        Row::new(["Time Authentication Status", "???", ""]),
    ];

    let widths = [
        Constraint::Percentage(50),
        Constraint::Percentage(35),
        Constraint::Percentage(15),
    ];

    let table = Table::new(rows, widths)
        .block(Block::bordered().title(Span::styled(
            "NAV-PVT",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )))
        .row_highlight_style(Style::default().fg(Color::Yellow))
        .header(
            Row::new(["Param", "Value", "Units"])
                .style(Style::new().bold())
                .bottom_margin(1)
                .fg(Color::Yellow),
        )
        .column_spacing(1)
        .style(Color::White);

    frame.render_widget(table, area);
}

fn render_esf_status(frame: &mut Frame, area: Rect, app: &mut App) {
    let vertical = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]);
    let [top, bottom] = vertical.areas(area);
    let horizontal = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
    let [top_left, top_right] = horizontal.areas(top);

    render_esf_alg_status(frame, top_left, app);
    render_esf_imu_alignment_status(frame, top_right, app);
    render_esf_sensor_status(frame, bottom, app);
}

fn render_esf_alg_status(frame: &mut Frame, area: Rect, app: &mut App) {
    let time_tag = format!("{:.3}", app.esf_alg_state.time_tag);
    let fusion_mode = match app.esf_alg_state.fusion_mode {
        EsfStatusFusionMode::Disabled => "DISABLED",
        EsfStatusFusionMode::Initializing => "INITIALIZING",
        EsfStatusFusionMode::Fusion => "FUSION",
        EsfStatusFusionMode::Suspended => "SUSPENDED",
        _ => "UNKNOWN",
    };

    let ins_status = match app.esf_alg_state.ins_status {
        EsfStatusInsInit::Off => "OFF",
        EsfStatusInsInit::Initialized => "INITIALIZED",
        EsfStatusInsInit::Initializing => "INITIALIZING",
    };

    let imu_status = match app.esf_alg_state.imu_status {
        EsfStatusImuInit::Off => "OFF",
        EsfStatusImuInit::Initialized => "INITIALIZED",
        EsfStatusImuInit::Initializing => "INITIALIZING",
    };

    let wt_status = match app.esf_alg_state.wheel_tick_sensor_status {
        EsfStatusWheelTickInit::Off => "OFF",
        EsfStatusWheelTickInit::Initialized => "INITIALIZED",
        EsfStatusWheelTickInit::Initializing => "INITIALIZING",
    };

    let mount_angle_status = match app.esf_alg_state.imu_mount_alignment_status {
        EsfStatusMountAngle::Off => "OFF",
        EsfStatusMountAngle::Initialized => "INITIALIZED",
        EsfStatusMountAngle::Initializing => "INITIALIZING",
    };

    let rows = [
        Row::new(["GPS Time Tag (s)", &time_tag]),
        Row::new(["Fusion Filter Mode", fusion_mode]),
        Row::new(["IMU Status", imu_status]),
        Row::new(["Wheel-tick Sensor Status", wt_status]),
        Row::new(["INS Status", ins_status]),
        Row::new(["IMU-mount Alignment Status", mount_angle_status]),
    ];

    let widths = [Constraint::Percentage(65), Constraint::Percentage(35)];

    let table = Table::new(rows, widths)
        .block(Block::bordered().title(Span::styled(
            "ESF-ALG-STATUS",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )))
        .row_highlight_style(Style::default().fg(Color::Yellow))
        .header(
            Row::new(["Name", "Status"])
                .style(Style::new().bold())
                .bottom_margin(1)
                .fg(Color::Yellow),
        )
        .column_spacing(1)
        .style(Color::White);

    frame.render_widget(table, area);
}

fn render_esf_imu_alignment_status(frame: &mut Frame, area: Rect, app: &mut App) {
    let time_tag = format!("{:.3}", app.esf_alg_imu_alignment_state.time_tag);
    let aligment_status = match app.esf_alg_imu_alignment_state.alignment_status {
        EsfAlgStatus::CoarseAlignment => "COARSE",
        EsfAlgStatus::FineAlignment => "FINE",
        EsfAlgStatus::UserDefinedAngles => "---",
        EsfAlgStatus::RollPitchAlignmentOngoing => "INITIALIZING", // "ROLL-PITCH-ONGOING",
        EsfAlgStatus::RollPitchYawAlignmentOngoing => "INITIALIZING", //"ROLL-PITCH-YAW-ONGOING",
    };

    let rows = [
        Row::new(["GPS Time Tag (s)", &time_tag]),
        Row::new([
            "Auto Alignment",
            if app.esf_alg_imu_alignment_state.auto_alignment {
                "ON"
            } else {
                "OFF"
            },
        ]),
        Row::new(["Alignment Status", aligment_status]),
        Row::new([
            "Angle Singularity",
            if app.esf_alg_imu_alignment_state.angle_singularity {
                "YES"
            } else {
                "NO"
            },
        ]),
        Row::new([
            Cell::from("Mounting-Roll (deg)"),
            Cell::from(format!("{:.4}", app.esf_alg_imu_alignment_state.roll)),
        ]),
        Row::new([
            Cell::from("Mounting-Pitch (deg)"),
            Cell::from(format!("{:.4}", app.esf_alg_imu_alignment_state.pitch)),
        ]),
        Row::new([
            Cell::from("Mounting-Yaw (deg)"),
            Cell::from(format!("{:.4}", app.esf_alg_imu_alignment_state.yaw)),
        ]),
    ];

    // Cell::from(sensor_type).style(Style::new().white()),

    let widths = [Constraint::Percentage(60), Constraint::Percentage(40)];

    let table = Table::new(rows, widths)
        .block(Block::bordered().title(Span::styled(
            "ESF-ALG-IMU-Alignment",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )))
        .row_highlight_style(Style::default().fg(Color::Yellow))
        .header(
            Row::new(["Name", "Status"])
                .style(Style::new().bold())
                .bottom_margin(1)
                .fg(Color::Yellow),
        )
        .column_spacing(1)
        .style(Color::White);

    frame.render_widget(table, area);
}

fn render_esf_sensor_status(frame: &mut Frame, area: Rect, app: &mut App) {
    let mut rows = vec![];

    for s in &app.esf_sensors_state.sensors {
        let sensor_type = match s.sensor_type {
            EsfSensorType::AccX => "Acc X",
            EsfSensorType::AccY => "Acc Y",
            EsfSensorType::AccZ => "Acc Z",
            EsfSensorType::GyroX => "Gyro X",
            EsfSensorType::GyroY => "Gyro Y",
            EsfSensorType::GyroZ => "Gyro Z",
            EsfSensorType::FrontLeftWheelTicks => "FL WheelTick",
            EsfSensorType::FrontRightWheelTicks => "FR WheelTick",
            EsfSensorType::RearLeftWheelTicks => "RL WheelTick",
            EsfSensorType::RearRightWheelTicks => "RR WheelTick",
            EsfSensorType::GyroTemp => "Gyro Temp",
            EsfSensorType::Speed => "Speed",
            EsfSensorType::SpeedTick => "Speed Tick",
            EsfSensorType::Unknown | EsfSensorType::None => "UNKNOWN",
        };

        let calibration_status = match s.calib_status {
            EsfSensorStatusCalibration::Calibrated => {
                Cell::from("CALIBRATED").style(Style::new().green())
            },

            EsfSensorStatusCalibration::NotCalibrated => {
                Cell::from("NOT CALIBRATED").style(Style::new().red())
            },
            EsfSensorStatusCalibration::Calibrating => {
                Cell::from("CALIBRATING").style(Style::new().yellow())
            },
        };

        let time_status = match s.time_status {
            EsfSensorStatusTime::NoData => "NoData",
            EsfSensorStatusTime::OnEventInput => "OnEventInput",
            EsfSensorStatusTime::TimeTagFromData => "DataTimeTag",
            EsfSensorStatusTime::OnReceptionFirstByte => "OnFirstByte",
        };

        let fault = if s.faults.contains(EsfSensorFaults::BAD_MEASUREMENT) {
            Cell::from("BAD MEASUREMENT").style(Style::new().yellow())
        } else if s.faults.contains(EsfSensorFaults::BAD_TIME_TAG) {
            Cell::from("BAD TIME TAG").style(Style::new().yellow())
        } else if s.faults.contains(EsfSensorFaults::MISSING_MEASUREMENT) {
            Cell::from("MISSING MEASUREMENT").style(Style::new().yellow())
        } else if s.faults.contains(EsfSensorFaults::NOISY_MEASUREMENT) {
            Cell::from("NOISY MEASUREMENT").style(Style::new().yellow())
        } else {
            Cell::from("").style(Style::new().white())
        };

        let row = Row::new(vec![
            Cell::from(sensor_type).style(Style::new().white()),
            calibration_status,
            Cell::from(time_status).style(Style::new().white()),
            Cell::from(s.freq.to_string()).style(Style::new().white()),
            fault,
        ]);
        rows.push(row);
    }

    let widths = [
        Constraint::Percentage(10),
        Constraint::Percentage(30),
        Constraint::Percentage(15),
        Constraint::Percentage(10),
        Constraint::Percentage(35),
    ];

    let table = Table::new(rows, widths)
        .block(Block::bordered().title(Span::styled(
            "ESF-SENSOR-STATUS",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )))
        .row_highlight_style(Style::default().fg(Color::Yellow))
        .header(
            Row::new(["Sensor", "Status", "Time", "Freq", "Faults"])
                .style(Style::new().bold())
                .bottom_margin(1)
                .fg(Color::Yellow),
        )
        .column_spacing(1)
        .style(Color::White);

    frame.render_widget(table, area);
}

fn render_monver(frame: &mut Frame, area: Rect, app: &mut App) {
    let extensions_src = app.mon_ver_state.extensions.clone();

    let mut extensions_lines = Vec::new();
    let mut extensions = extensions_src.to_string();
    let mut extensions = if let Some(p) = extensions.find("FWVER") {
        let suffix = extensions.split_off(p);
        extensions_lines.push(Line::from(extensions));
        suffix
    } else {
        String::default()
    };

    let mut extensions = if let Some(p) = extensions.find("PROTVER") {
        let suffix = extensions.split_off(p);
        extensions_lines.push(Line::from(extensions));
        suffix
    } else {
        String::default()
    };

    let mut extensions = if let Some(p) = extensions.find("MOD") {
        let suffix = extensions.split_off(p);
        extensions_lines.push(Line::from(extensions));
        suffix
    } else {
        String::default()
    };

    let mut extensions = if let Some(p) = extensions.find("FIS") {
        let suffix = extensions.split_off(p);
        extensions_lines.push(Line::from(extensions));
        suffix
    } else {
        String::default()
    };

    let extensions = if let Some(p) = extensions.find(")") {
        let suffix = extensions.split_off(p + 1);
        extensions_lines.push(Line::from(extensions));
        suffix
    } else {
        String::default()
    };

    // Remaining content of extensions string
    extensions_lines.push(Line::from(extensions));

    let software_version = std::str::from_utf8(&app.mon_ver_state.software_version).unwrap();
    let hardware_version = std::str::from_utf8(&app.mon_ver_state.hardware_version).unwrap();

    let mut text = vec![
        Line::from(Span::styled(
            "Software Version",
            Style::default().fg(Color::Red),
        )),
        Line::from(vec![Span::from(" "), Span::from(software_version)]),
        Line::from(""),
        Line::from(Span::styled(
            "Hardware Version",
            Style::default().fg(Color::Red),
        )),
        Line::from(vec![Span::raw(""), Span::from(hardware_version)]),
        Line::from(""),
        Line::from(Span::styled(
            "Extensions",
            Style::default().fg(Color::Yellow),
        )),
    ];
    text.append(&mut extensions_lines);

    let mut raw_extensions = vec![
        Line::from(""),
        Line::from("Extensions as raw string:"),
        Line::from(extensions_src),
    ];

    text.append(&mut raw_extensions);

    let block = Block::bordered().title(Span::styled(
        "MON-VERSION",
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    ));
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn draw_map(frame: &mut Frame, app: &mut App, area: Rect) {
    // let pos = app.pvt_state.lat
    let map = Canvas::default()
        .block(Block::bordered().title("World"))
        .paint(|ctx| {
            ctx.draw(&Map {
                color: Color::White,
                resolution: MapResolution::High,
            });
            ctx.layer();
            ctx.draw(&Circle {
                x: app.pvt_state.lon,
                y: app.pvt_state.lat,
                radius: 10.0,
                color: Color::Green,
            });
            ctx.print(
                app.pvt_state.lon,
                app.pvt_state.lat,
                Span::styled("X", Style::default().fg(Color::Green)),
            );
        })
        .marker(symbols::Marker::Braille)
        .x_bounds([-180.0, 180.0])
        .y_bounds([-90.0, 90.0]);
    frame.render_widget(map, area);
}
