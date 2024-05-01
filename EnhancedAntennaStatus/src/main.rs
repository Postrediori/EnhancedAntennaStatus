mod modem_utils;
use modem_utils::{*};

mod bar_plot_widget;
use bar_plot_widget::{*};

mod res;
use res::IconsAssets;

use fltk::{*, prelude::{*}};
use std::thread;
use std::time::Duration;
use std::{cell::RefCell, rc::Rc};

const POLLER_TIMEOUT: u64 = 2_000;

#[derive(Clone, Copy)]
enum Message {
    GetInfo,
    StartStopPolling,
    Poller,
    ReceivedInfo(ModemStatus),
    SetMode(NetworkMode),
    SetWcdmaInfo(WcdmaSignalInfo),
    SetLteInfo(LteSignalInfo),
}

const PARAM_COLOR: enums::Color = enums::Color::DarkRed;

#[macro_export]
macro_rules! set_param_label {
    ($widget:tt) => {
        $widget.set_text_color(PARAM_COLOR);
        $widget.set_color(enums::Color::Background);
        $widget.set_frame(enums::FrameType::FlatBox);
    }
}

#[macro_export]
macro_rules! set_frame_style {
    ($widget:tt) => {
        $widget.set_label_type(enums::LabelType::Embossed);
        $widget.set_align(enums::Align::LeftBottom | enums::Align::Inside);
        $widget.set_frame(enums::FrameType::EngravedFrame);
        $widget.set_color(enums::Color::Dark3);
    }
}

#[macro_export]
macro_rules! add_flex_spacer {
    ($flex:tt, $size:expr) => {
        let spacer = frame::Frame::default();
        $flex.fixed(&spacer, $size);
    }
}

fn main() {
    let app = app::App::default();

    let (tx, rx) = app::channel::<Message>();

    const WIDTH: i32 = 480;
    const HEIGHT: i32 = 650;
    let mut wnd = window::Window::default()
        .with_size(WIDTH, HEIGHT).with_label("Enhanced Antenna Status");

    let mut main_flex = group::Flex::default_fill()
        .column();
    main_flex.set_margin(10);
    main_flex.set_spacing(5);

    let (mut host_input, mut connect_button) = {
        let mut row = group::Flex::default_fill()
            .row();

        add_flex_spacer!(row, 115);

        let host_input = input::Input::default()
            .with_label("Modem Address:");
    
        let connect_button = button::Button::default()
            .with_label("Start Poll");
        row.fixed(&connect_button, 75);

        add_flex_spacer!(row, 55);

        row.end();
        main_flex.fixed(&row, 25);

        (host_input, connect_button)
    };

    /*
     * General info
     */
    let mut info_flex = group::Flex::default_fill()
        .column()
        .with_label("General Info");
    set_frame_style!(info_flex);
    info_flex.set_margin(5);

    add_flex_spacer!(info_flex, 10);

    let (mut network_mode_label, mut rssi_label) = {
        let mut row = group::Flex::default_fill()
            .row();
        row.set_spacing(5);

        add_flex_spacer!(row, 110);

        let mut network_mode_label = output::Output::default()
            .with_label("Network mode:");
        set_param_label!(network_mode_label);

        add_flex_spacer!(row, 50);

        let mut rssi_label = output::Output::default()
            .with_label("RSSI:");
        set_param_label!(rssi_label);

        row.end();
        info_flex.fixed(&row, 20);

        (network_mode_label, rssi_label)
    };

    let (mut plmn_label, mut band_label) = {
        let mut row = group::Flex::default_fill()
            .row();
        row.set_spacing(5);

        add_flex_spacer!(row, 75);

        let mut plmn_label = output::Output::default()
            .with_label("PLMN:");
        set_param_label!(plmn_label);

        add_flex_spacer!(row, 50);

        let mut band_label = output::Output::default()
            .with_label("Band:");
        set_param_label!(band_label);

        row.end();
        info_flex.fixed(&row, 20);

        (plmn_label, band_label)
    };

    let mut cellid_label = {
        let mut row = group::Flex::default_fill()
            .row();
        row.set_spacing(5);

        add_flex_spacer!(row, 75);

        let mut cellid_label = output::Output::default()
            .with_label("Cell ID:");
        set_param_label!(cellid_label);

        row.end();
        info_flex.fixed(&row, 20);

        cellid_label
    };

    info_flex.end();
    main_flex.fixed(&info_flex, 100);

    /*
     * Modem info
     */
    let mut modem_info_flex = group::Flex::default_fill()
        .column()
        .with_label("Modem Info");
    set_frame_style!(modem_info_flex);
    modem_info_flex.set_margin(5);

    add_flex_spacer!(modem_info_flex, 10);

    let (mut manufacturer_label, mut model_label) = {
        let mut row = group::Flex::default_fill()
            .row();
        row.set_spacing(5);

        add_flex_spacer!(row, 110);

        let mut manufacturer_label = output::Output::default()
            .with_label("Manufacturer:");
        set_param_label!(manufacturer_label);

        add_flex_spacer!(row, 75);

        let mut model_label = output::Output::default()
            .with_label("Model:");
        set_param_label!(model_label);

        row.end();
        modem_info_flex.fixed(&row, 20);

        (manufacturer_label, model_label)
    };

    let (mut battery_percent_label, mut battery_status_label) = {
        let mut row = group::Flex::default_fill()
            .row();
        row.set_spacing(5);

        add_flex_spacer!(row, 75);

        let mut battery_percent_label = output::Output::default()
            .with_label("Battery:");
        set_param_label!(battery_percent_label);

        add_flex_spacer!(row, 50);

        let mut battery_status_label = output::Output::default()
            .with_label("Charge:");
        set_param_label!(battery_status_label);

        row.end();
        modem_info_flex.fixed(&row, 20);

        (battery_percent_label, battery_status_label)
    };

    let (mut device_temp_label, mut battery_temp_label) = {
        let mut row = group::Flex::default_fill()
            .row();
        row.set_spacing(5);

        add_flex_spacer!(row, 90);

        let mut device_temp_label = output::Output::default()
            .with_label("Device Temp:");
        set_param_label!(device_temp_label);

        add_flex_spacer!(row, 75);

        let mut battery_temp_label = output::Output::default()
            .with_label("Battery Temp:");
        set_param_label!(battery_temp_label);

        row.end();
        modem_info_flex.fixed(&row, 20);

        (device_temp_label, battery_temp_label)
    };

    info_flex.end();
    main_flex.fixed(&modem_info_flex, 105);

    /*
     * WCDMA signal status
     */
    let mut wcdma_flex = group::Flex::default_fill()
        .column()
        .with_label("3G");
    set_frame_style!(wcdma_flex);
    wcdma_flex.set_margin(5);

    add_flex_spacer!(wcdma_flex, 10);

    let (mut wcdma_nb_cc_label, mut wcdma_rnc_label, mut wcdma_sc_label) = {
        let mut row = group::Flex::default_fill()
            .row();
        row.set_spacing(5);

        add_flex_spacer!(row, 75);

        let mut wcdma_nb_cc_label = output::Output::default()
            .with_label("NB / Cell:");
        set_param_label!(wcdma_nb_cc_label);

        add_flex_spacer!(row, 75);

        let mut wcdma_rnc_label = output::Output::default()
            .with_label("RNC-ID:");
        set_param_label!(wcdma_rnc_label);

        add_flex_spacer!(row, 75);

        let mut wcdma_sc_label = output::Output::default()
            .with_label("SC:");
        set_param_label!(wcdma_sc_label);

        row.end();
        wcdma_flex.fixed(&row, 15);

        (wcdma_nb_cc_label, wcdma_rnc_label, wcdma_sc_label)
    };

    let mut rscp_label = {
        let mut row = group::Flex::default_fill()
            .row();

        add_flex_spacer!(row, 75);
        
        let mut rscp_label = output::Output::default()
            .with_label("RSCP:");
        set_param_label!(rscp_label);

        row.end();
        wcdma_flex.fixed(&row, 15);

        rscp_label
    };

    let mut rscp_plot = BarPlotWidget::new();
    rscp_plot.set_range(-100, -70);

    let mut ecio_label = {
        let mut row = group::Flex::default_fill()
            .row();

        add_flex_spacer!(row, 75);
        
        let mut ecio_label = output::Output::default()
            .with_label("EC/IO:");
        set_param_label!(ecio_label);

        row.end();
        wcdma_flex.fixed(&row, 15);

        ecio_label
    };

    let mut ecio_plot = BarPlotWidget::new();
    ecio_plot.set_range(-10, -2);

    wcdma_flex.end();

    /*
     * LTE signal status
     */
    let mut lte_flex = group::Flex::default_fill()
        .column()
        .with_label("LTE");
    set_frame_style!(lte_flex);
    lte_flex.set_margin(5);

    add_flex_spacer!(lte_flex, 10);

    let (mut lte_enb_cc_label, mut lte_pci_label) = {
        let mut row = group::Flex::default_fill()
            .row();
        row.set_spacing(5);

        add_flex_spacer!(row, 75);

        let mut lte_enb_cc_label = output::Output::default()
            .with_label("eNB / Cell:");
        set_param_label!(lte_enb_cc_label);

        add_flex_spacer!(row, 75);

        let mut lte_pci_label = output::Output::default()
            .with_label("PCI:");
        set_param_label!(lte_pci_label);

        row.end();
        lte_flex.fixed(&row, 20);

        (lte_enb_cc_label, lte_pci_label)
    };

    let mut rsrq_label = {
        let mut row = group::Flex::default_fill()
            .row();

        add_flex_spacer!(row, 75);
        
        let mut rsrq_label = output::Output::default()
            .with_label("RSRQ:");
        set_param_label!(rsrq_label);

        row.end();
        lte_flex.fixed(&row, 15);

        rsrq_label
    };

    let mut rsrq_plot = BarPlotWidget::new();
    rsrq_plot.set_range(-16, -3);

    let mut rsrp_label = {
        let mut row = group::Flex::default_fill()
            .row();

        add_flex_spacer!(row, 75);
        
        let mut rsrp_label = output::Output::default()
            .with_label("RSRP:");
        set_param_label!(rsrp_label);

        row.end();
        lte_flex.fixed(&row, 15);

        rsrp_label
    };

    let mut rsrp_plot = BarPlotWidget::new();
    rsrp_plot.set_range(-130, -60);

    let mut sinr_label = {
        let mut row = group::Flex::default_fill()
            .row();

        add_flex_spacer!(row, 75);
        
        let mut sinr_label = output::Output::default()
            .with_label("SINR:");
        set_param_label!(sinr_label);

        row.end();
        lte_flex.fixed(&row, 15);

        sinr_label
    };

    let mut sinr_plot = BarPlotWidget::new();
    sinr_plot.set_range(0, 24);
    
    lte_flex.end();

    /*
     * Final setup of window
     */
    main_flex.end();

    if let Some(img) = IconsAssets::get("EnhancedAntennaStatus32.png") {
        if let Ok(img) = fltk::image::PngImage::from_data(img.data.as_ref()) {
            wnd.set_icon(Some(img));
        }
    }

    wnd.end();
    wnd.make_resizable(true);

    /*
     * Initial state of UI
     */
    host_input.set_value("192.168.1.1");
    connect_button.emit(tx, Message::StartStopPolling);

    wcdma_flex.hide();
    lte_flex.hide();

    wnd.show();

    /*
     * Variables
     */
    let run_poller = false;
    let run_poller = Rc::from(RefCell::from(run_poller));

    let host_address = "".to_string();
    let host_address = Rc::from(RefCell::from(host_address));

    let mut current_pci = -1;
    let mut current_mode = NetworkMode::Unknown;

    let mut jh_poller: Option<std::thread::JoinHandle<()>> = None;
    let mut jh_getinfo: Option<std::thread::JoinHandle<()>> = None;

    /*
     * Run main event loop
     */
    {
        let run_poller = run_poller.clone();
        let host_address = host_address.clone();

        while app.wait() {
            let mut run_poller = run_poller.borrow_mut();
            let mut host_address = host_address.borrow_mut();
            if let Some(msg) = rx.recv() {
                match msg {
                    Message::StartStopPolling => {
                        *run_poller = !*run_poller;
                        if *run_poller {
                            *host_address = host_input.value();
                            tx.send(Message::Poller);

                            host_input.deactivate();
                            connect_button.set_label("Stop Poll");
                        }
                        else {
                            host_input.activate();
                            connect_button.set_label("Start Poll");
                        }
                    },
                    Message::GetInfo => {
                        println!("Connecting to host {}", *host_address);

                        // Wait for existing thread to stop
                        if let Some(jh) = jh_getinfo {
                            jh.join().unwrap();
                        }

                        jh_getinfo = Some(thread::spawn({
                            let host_address = host_address.clone();
                            move || {
                                if let Some(modem_info) = NetgearParser::get_info(host_address.as_str()) {
                                    tx.send(Message::ReceivedInfo(modem_info))
                                }
                            }
                        }));
                    },
                    Message::ReceivedInfo(info) => {
                        println!("{info}\n");

                        network_mode_label.set_value(info.get_mode().as_str());
            
                        rssi_label.set_value(format!("{} dBm", info.rssi).as_str());
            
                        plmn_label.set_value(&info.get_plmn());
            
                        band_label.set_value(&info.get_band());
            
                        let (cell_id_hex, cell_id) = info.get_cell_id_hex_and_dec();
                        cellid_label.set_value(format!("{cell_id_hex}/{cell_id}").as_str());
            
                        if current_mode != info.mode {
                            tx.send(Message::SetMode(info.mode));
                        }

                        match info.signal_info {
                            SignalInfo::Wcdma(wcdma_info) => tx.send(Message::SetWcdmaInfo(wcdma_info)),
                            SignalInfo::Lte(lte_info) => tx.send(Message::SetLteInfo(lte_info)),
                            _=> { }
                        }

                        // Modem model
                        let (manufacturer, model) = info.get_manufacturer_and_model();
                        manufacturer_label.set_value(&manufacturer);
                        model_label.set_value(&model);

                        // Battery info
                        let (battery_percent, battery_status) = info.get_battery_percent_and_status();
                        
                        let battery_percent = format!("{}%", battery_percent);
                        battery_percent_label.set_value(&battery_percent);
                        battery_status_label.set_value(&battery_status);
                        
                        // Temperature
                        device_temp_label.set_value(format!("{}°C", info.device_temp).as_str());
                        battery_temp_label.set_value(format!("{}°C", info.battery_temp).as_str());

                        wnd.redraw();
                    },
                    Message::Poller => {
                        if *run_poller {
                            // Stop previous thread
                            if let Some(jh) = jh_poller {
                                jh.join().unwrap();
                            }

                            // Make step and schedule next 'Running' poll
                            jh_poller = Some(thread::spawn(move || {
                                tx.send(Message::GetInfo);
                                thread::sleep(Duration::from_millis(POLLER_TIMEOUT));
                                tx.send(Message::Poller);
                            }));
                        }
                    },
                    Message::SetMode(mode) => {
                        // Clean WCDMA status
                        wcdma_flex.hide();

                        wcdma_sc_label.set_value("");
                        wcdma_rnc_label.set_value("");
                        wcdma_nb_cc_label.set_value("");

                        rscp_label.set_value("");
                        ecio_label.set_value("");

                        rscp_plot.clear_history();
                        ecio_plot.clear_history();

                        // Clean LTE status
                        lte_flex.hide();

                        lte_enb_cc_label.set_value("");
                        lte_pci_label.hide();

                        rsrp_label.set_value("");
                        rsrq_label.set_value("");
                        sinr_label.set_value("");

                        rsrp_plot.clear_history();
                        rsrq_plot.clear_history();
                        sinr_plot.clear_history();

                        // Set active mode
                        match mode {
                            NetworkMode::Lte => {
                                lte_flex.show();
                            }
                            NetworkMode::Wcdma => {
                                wcdma_flex.show();
                            }
                            _ => { }
                        };

                        main_flex.layout();

                        current_mode = mode;
                    },
                    Message::SetWcdmaInfo(wcdma_info) => {
                        wcdma_sc_label.set_value(&wcdma_info.psc.to_string());
                        wcdma_rnc_label.set_value(&wcdma_info.rnc.to_string());
                        wcdma_nb_cc_label.set_value(format!("{}/{}", wcdma_info.nb, wcdma_info.cc).as_str());

                        rscp_label.set_value(format!("{} dBm", wcdma_info.rscp).as_str());
                        ecio_label.set_value(format!("{} dB", wcdma_info.ecio).as_str());

                        rscp_plot.push_value(wcdma_info.rscp);
                        ecio_plot.push_value(wcdma_info.ecio);
                    },
                    Message::SetLteInfo(lte_info) => {
                        if current_pci != lte_info.pci {
                            if lte_info.pci == -1 {
                                lte_pci_label.hide();
                            }
                            else {
                                lte_pci_label.show();
                                lte_pci_label.set_value(&lte_info.pci.to_string());
                            }
                            lte_flex.layout();
                            current_pci = lte_info.pci;
                        }
    
                        lte_enb_cc_label.set_value(format!("{}/{}", lte_info.enb, lte_info.id).as_str());

                        rsrp_label.set_value(format!("{} dB", lte_info.rsrp).as_str());
                        rsrq_label.set_value(format!("{} dBm", lte_info.rsrq).as_str());
                        sinr_label.set_value(format!("{} dB", lte_info.sinr).as_str());

                        rsrp_plot.push_value(lte_info.rsrp);
                        rsrq_plot.push_value(lte_info.rsrq);
                        sinr_plot.push_value(lte_info.sinr);
                    }
                }
            }
        }
    }

    // Stop all threads
    {
        *run_poller.borrow_mut() = false;

        if let Some(jh) = jh_poller {
            jh.join().unwrap();
        }

        if let Some(jh) = jh_getinfo {
            jh.join().unwrap();
        }
    }
}
