use fltk::{*, prelude::*};

use crate::res::IconsAssets;
use crate::bar_plot_widget::*;
use crate::modem_utils::*;
use crate::bandwidth_utils::*;
use crate::utils::*;

/*
 * Poll timeout
 */
const POLL_TIMEOUT_VALUES: [(u64, &'static str); 7] = [
    (1, "1 sec"),
    (2, "2 sec"),
    (5, "5 sec"),
    (10, "10 sec"),
    (15, "15 sec"),
    (30, "30 sec"),
    (60, "1 min"),
];

/*
 * Macro for UI
 */
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

/*
 * MainWindow
 */
pub struct MainWindow {
    current_pci: ValueChangeObserver::<i64>,
    current_mode: ValueChangeObserver::<NetworkMode>,
    pub wnd: window::Window,
    main_group: group::Flex,
    pub host_input: input::Input,
    pub connect_button: button::Button,
    timeout_choice: menu::Choice,
    pub close_button: button::Button,
    network_mode_label: output::Output,
    rssi_label: output::Output,
    plmn_label: output::Output,
    band_label: output::Output,
    cellid_label: output::Output,
    manufacturer_label: output::Output,
    model_label: output::Output,
    battery_percent_label: output::Output,
    battery_status_label: output::Output,
    device_temp_label: output::Output,
    battery_temp_label: output::Output,
    wcdma_group: group::Flex,
    wcdma_nb_cc_label: output::Output,
    wcdma_rnc_label: output::Output,
    wcdma_sc_label: output::Output,
    rscp_label: output::Output,
    rscp_plot: BarPlotWidget,
    ecio_label: output::Output,
    ecio_plot: BarPlotWidget,
    lte_group: group::Flex,
    lte_enb_cc_label: output::Output,
    lte_pci_label: output::Output,
    rsrq_label: output::Output,
    rsrq_plot: BarPlotWidget,
    rsrp_label: output::Output,
    rsrp_plot: BarPlotWidget,
    sinr_label: output::Output,
    sinr_plot: BarPlotWidget,
    dl_label: output::Output,
    ul_label: output::Output,
    dlul_plot: DlUlBarPlotWidget,
    error_label: frame::Frame,
}

impl MainWindow {
    pub fn new(width: i32, height: i32) -> Self {
        let current_pci = ValueChangeObserver::<i64>::new();
        let current_mode = ValueChangeObserver::<NetworkMode>::new();

        let mut wnd = window::Window::default()
            .with_size(width, height)
            .with_label("Enhanced Antenna Status");

        let mut main_group = group::Flex::default_fill()
            .column();
        main_group.set_margin(10);
        main_group.set_spacing(5);
    
        let (mut host_input, connect_button, timeout_choice) = {
            let mut row = group::Flex::default_fill()
                .row();
    
            add_flex_spacer!(row, 115);
    
            let host_input = input::Input::default()
                .with_label("Modem Address:");
        
            let connect_button = button::Button::default()
                .with_label("Start Poll");
            row.fixed(&connect_button, 75);
    
            add_flex_spacer!(row, 105);

            let mut timeout_choice = menu::Choice::default()
                .with_label("Poll timeout:");
            row.fixed(&timeout_choice, 75);

            for pt in POLL_TIMEOUT_VALUES {
                timeout_choice.add_choice(pt.1);
            }
            timeout_choice.set_value(1);
    
            row.end();
            main_group.fixed(&row, 25);
    
            (host_input, connect_button, timeout_choice)
        };
    
        let info_group_container = group::Flex::default_fill()
            .row();
    
        /*
         * General info
         */
        let mut info_group = group::Flex::default_fill()
            .column()
            .with_label("General Info");
        set_frame_style!(info_group);
        info_group.set_margin(5);
    
        add_flex_spacer!(info_group, 10);
    
        let (network_mode_label, rssi_label) = {
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
            info_group.fixed(&row, 20);
    
            (network_mode_label, rssi_label)
        };
    
        let (plmn_label, band_label) = {
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
            info_group.fixed(&row, 20);
    
            (plmn_label, band_label)
        };
    
        let cellid_label = {
            let mut row = group::Flex::default_fill()
                .row();
            row.set_spacing(5);
    
            add_flex_spacer!(row, 75);
    
            let mut cellid_label = output::Output::default()
                .with_label("Cell ID:");
            set_param_label!(cellid_label);
    
            row.end();
            info_group.fixed(&row, 20);
    
            cellid_label
        };
    
        info_group.end();
    
        /*
         * Modem info
         */
        let mut modem_info_group = group::Flex::default_fill()
            .column()
            .with_label("Modem Info");
        set_frame_style!(modem_info_group);
        modem_info_group.set_margin(5);
    
        add_flex_spacer!(modem_info_group, 10);
    
        let (manufacturer_label, model_label) = {
            let mut row = group::Flex::default_fill()
                .row();
            row.set_spacing(5);
    
            add_flex_spacer!(row, 110);
    
            let mut manufacturer_label = output::Output::default()
                .with_label("Manufacturer:");
            set_param_label!(manufacturer_label);
    
            add_flex_spacer!(row, 60);
    
            let mut model_label = output::Output::default()
                .with_label("Model:");
            set_param_label!(model_label);
    
            row.end();
            modem_info_group.fixed(&row, 20);
    
            (manufacturer_label, model_label)
        };
    
        let (battery_percent_label, battery_status_label) = {
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
            modem_info_group.fixed(&row, 20);
    
            (battery_percent_label, battery_status_label)
        };
    
        let (device_temp_label, battery_temp_label) = {
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
            modem_info_group.fixed(&row, 20);
    
            (device_temp_label, battery_temp_label)
        };
    
        info_group.end();
    
        info_group_container.end();
        main_group.fixed(&info_group_container, 105);
    
        let plot_group_container = group::Flex::default_fill()
            .row();
    
        /*
         * WCDMA signal status
         */
        let mut wcdma_group = group::Flex::default_fill()
            .column()
            .with_label("3G");
        set_frame_style!(wcdma_group);
        wcdma_group.set_margin(5);
    
        add_flex_spacer!(wcdma_group, 10);
    
        let (wcdma_nb_cc_label, wcdma_rnc_label, wcdma_sc_label) = {
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
            wcdma_group.fixed(&row, 15);
    
            (wcdma_nb_cc_label, wcdma_rnc_label, wcdma_sc_label)
        };
    
        let rscp_label = {
            let mut row = group::Flex::default_fill()
                .row();
    
            add_flex_spacer!(row, 75);
            
            let mut rscp_label = output::Output::default()
                .with_label("RSCP:");
            set_param_label!(rscp_label);
    
            row.end();
            wcdma_group.fixed(&row, 15);
    
            rscp_label
        };
    
        let mut rscp_plot = BarPlotWidget::new();
        rscp_plot.set_range(-100, -70);
    
        let ecio_label = {
            let mut row = group::Flex::default_fill()
                .row();
    
            add_flex_spacer!(row, 75);
            
            let mut ecio_label = output::Output::default()
                .with_label("EC/IO:");
            set_param_label!(ecio_label);
    
            row.end();
            wcdma_group.fixed(&row, 15);
    
            ecio_label
        };
    
        let mut ecio_plot = BarPlotWidget::new();
        ecio_plot.set_range(-10, -2);
    
        wcdma_group.end();
    
        /*
         * LTE signal status
         */
        let mut lte_group = group::Flex::default_fill()
            .column()
            .with_label("LTE");
        set_frame_style!(lte_group);
        lte_group.set_margin(5);
    
        add_flex_spacer!(lte_group, 10);
    
        let (lte_enb_cc_label, lte_pci_label) = {
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
            lte_group.fixed(&row, 20);
    
            (lte_enb_cc_label, lte_pci_label)
        };
    
        let rsrq_label = {
            let mut row = group::Flex::default_fill()
                .row();
    
            add_flex_spacer!(row, 75);
            
            let mut rsrq_label = output::Output::default()
                .with_label("RSRQ:");
            set_param_label!(rsrq_label);
    
            row.end();
            lte_group.fixed(&row, 15);
    
            rsrq_label
        };
    
        let mut rsrq_plot = BarPlotWidget::new();
        rsrq_plot.set_range(-16, -3);
    
        let rsrp_label = {
            let mut row = group::Flex::default_fill()
                .row();
    
            add_flex_spacer!(row, 75);
            
            let mut rsrp_label = output::Output::default()
                .with_label("RSRP:");
            set_param_label!(rsrp_label);
    
            row.end();
            lte_group.fixed(&row, 15);
    
            rsrp_label
        };
    
        let mut rsrp_plot = BarPlotWidget::new();
        rsrp_plot.set_range(-130, -60);
    
        let sinr_label = {
            let mut row = group::Flex::default_fill()
                .row();
    
            add_flex_spacer!(row, 75);
            
            let mut sinr_label = output::Output::default()
                .with_label("SINR:");
            set_param_label!(sinr_label);
    
            row.end();
            lte_group.fixed(&row, 15);
    
            sinr_label
        };
    
        let mut sinr_plot = BarPlotWidget::new();
        sinr_plot.set_range(0, 24);
        
        lte_group.end();
    
        /*
         * Bandwidth
         */
        let mut bandwidth_group = group::Flex::default_fill()
            .column()
            .with_label("Bandwidth");
        set_frame_style!(bandwidth_group);
        bandwidth_group.set_margin(5);
    
        add_flex_spacer!(bandwidth_group, 10);
    
        let (dl_label, ul_label) = {
            let mut row = group::Flex::default_fill()
                .row();
            row.set_spacing(5);
    
            add_flex_spacer!(row, 75);
    
            let mut dl_label = output::Output::default()
                .with_label("Download:");
            set_param_label!(dl_label);
    
            add_flex_spacer!(row, 75);
    
            let mut ul_label = output::Output::default()
                .with_label("Upload:");
            set_param_label!(ul_label);
    
            row.end();
            bandwidth_group.fixed(&row, 20);
    
            (dl_label, ul_label)
        };
    
        let dlul_plot = DlUlBarPlotWidget::new();
    
        bandwidth_group.end();
    
        plot_group_container.end();
    
        /*
         * Footer
         */
        let mut footer_group = group::Flex::default_fill()
            .row();

        let mut error_label = frame::Frame::default();
        error_label.set_label_color(PARAM_COLOR);
        error_label.set_color(enums::Color::Background);
        error_label.set_frame(enums::FrameType::FlatBox);
        error_label.set_align(enums::Align::Left | enums::Align::Inside);

        let close_button = button::Button::default()
            .with_label("Close");

        footer_group.fixed(&close_button, 75);
        footer_group.end();
    
        main_group.fixed(&footer_group, 25);
    
        /*
         * Final setup of window
         */
        main_group.end();
    
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

        wcdma_group.hide();
        lte_group.hide();

        Self {
            current_pci, current_mode,
            wnd,
            main_group,
            host_input,
            connect_button, timeout_choice, close_button,
            network_mode_label, rssi_label,
            plmn_label, band_label, cellid_label,
            manufacturer_label, model_label,
            battery_percent_label, battery_status_label,
            device_temp_label, battery_temp_label,
            wcdma_group, wcdma_nb_cc_label, wcdma_rnc_label, wcdma_sc_label,
            rscp_label, rscp_plot, ecio_label, ecio_plot,
            lte_group, lte_enb_cc_label, lte_pci_label,
            rsrp_label, rsrp_plot, rsrq_label, rsrq_plot, sinr_label, sinr_plot,
            dl_label, ul_label, dlul_plot,
            error_label,
        }
    }
    pub fn set_info(&mut self, info: ModemStatus) {
        self.network_mode_label.set_value(info.get_mode().as_str());
            
        self.rssi_label.set_value(format!("{} dBm", info.rssi).as_str());

        self.plmn_label.set_value(&info.get_plmn());

        self.band_label.set_value(&info.get_band());

        let (cell_id_hex, cell_id) = info.get_cell_id_hex_and_dec();
        self.cellid_label.set_value(format!("{cell_id_hex}/{cell_id}").as_str());

        if self.current_mode.update_and_check_if_changed(info.mode) {
            self.set_mode(info.mode);
        }

        match info.signal_info {
            SignalInfo::Wcdma(wcdma_info) => self.set_wcdma_info(wcdma_info),
            SignalInfo::Lte(lte_info) => self.set_lte_info(lte_info),
            _=> { }
        }

        // Modem model
        let (manufacturer, model) = info.get_manufacturer_and_model();
        self.manufacturer_label.set_value(&manufacturer);
        self.model_label.set_value(&model);

        // Battery info
        let (battery_percent, battery_status) = info.get_battery_percent_and_status();
        
        let battery_percent = format!("{}%", battery_percent);
        self.battery_percent_label.set_value(&battery_percent);
        self.battery_status_label.set_value(&battery_status);
        
        // Temperature
        self.device_temp_label.set_value(format!("{}°C", info.device_temp).as_str());
        self.battery_temp_label.set_value(format!("{}°C", info.battery_temp).as_str());

        self.wnd.redraw();
    }
    fn set_mode(&mut self, mode: NetworkMode) {
        // Clean WCDMA status
        self.wcdma_group.hide();

        self.wcdma_sc_label.set_value("");
        self.wcdma_rnc_label.set_value("");
        self.wcdma_nb_cc_label.set_value("");

        self.rscp_label.set_value("");
        self.ecio_label.set_value("");

        self.rscp_plot.clear_history();
        self.ecio_plot.clear_history();

        // Clean LTE status
        self.lte_group.hide();

        self.lte_enb_cc_label.set_value("");
        self.lte_pci_label.hide();

        self.rsrp_label.set_value("");
        self.rsrq_label.set_value("");
        self.sinr_label.set_value("");

        self.rsrp_plot.clear_history();
        self.rsrq_plot.clear_history();
        self.sinr_plot.clear_history();

        self.dlul_plot.clear_history();

        // Set active mode
        match mode {
            NetworkMode::Lte => {
                self.lte_group.show();
            }
            NetworkMode::Wcdma => {
                self.wcdma_group.show();
            }
            _ => { }
        };

        self.main_group.layout();
        self.wnd.redraw();
    }
    fn set_wcdma_info(&mut self, wcdma_info: WcdmaSignalInfo) {
        self.wcdma_sc_label.set_value(&wcdma_info.psc.to_string());
        self.wcdma_rnc_label.set_value(&wcdma_info.rnc.to_string());
        self.wcdma_nb_cc_label.set_value(format!("{}/{}", wcdma_info.nb, wcdma_info.cc).as_str());

        self.rscp_label.set_value(format!("{} dBm", wcdma_info.rscp).as_str());
        self.ecio_label.set_value(format!("{} dB", wcdma_info.ecio).as_str());

        self.rscp_plot.push_value(wcdma_info.rscp);
        self.ecio_plot.push_value(wcdma_info.ecio);
    }
    fn set_lte_info(&mut self, lte_info: LteSignalInfo) {
        if self.current_pci.update_and_check_if_changed(lte_info.pci) {
            if lte_info.pci == -1 {
                self.lte_pci_label.hide();
            }
            else {
                self.lte_pci_label.show();
                self.lte_pci_label.set_value(&lte_info.pci.to_string());
            }
            self.lte_group.layout();
        }

        self.lte_enb_cc_label.set_value(format!("{}/{}", lte_info.enb, lte_info.id).as_str());

        self.rsrp_label.set_value(format!("{} dB", lte_info.rsrp).as_str());
        self.rsrq_label.set_value(format!("{} dBm", lte_info.rsrq).as_str());
        self.sinr_label.set_value(format!("{} dB", lte_info.sinr).as_str());

        self.rsrp_plot.push_value(lte_info.rsrp);
        self.rsrq_plot.push_value(lte_info.rsrq);
        self.sinr_plot.push_value(lte_info.sinr);
    }
    pub fn set_bandwidth_data(&mut self, dlul: (i64, i64)) {
        let dl_str = format_bandwidth(dlul.0);
        let ul_str = format_bandwidth(dlul.1);

        self.dl_label.set_value(&dl_str);
        self.ul_label.set_value(&ul_str);

        self.dlul_plot.push_value(dlul);
    }
    pub fn start_poll(&mut self) {
        self.host_input.deactivate();
        self.connect_button.set_label("Stop Poll");
        self.timeout_choice.deactivate();
    }
    pub fn stop_poll(&mut self) {
        self.host_input.activate();
        self.connect_button.set_label("Start Poll");
        self.timeout_choice.activate();
    }
    pub fn set_error(&mut self, s: &str) {
        self.error_label.set_label(s);
    }
    pub fn get_poll_timeout(&self) -> u64 {
        let i = self.timeout_choice.value() as usize;
        POLL_TIMEOUT_VALUES[i].0
    }
}
