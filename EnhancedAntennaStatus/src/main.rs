#![allow(clippy::cast_sign_loss)]
#![allow(clippy::too_many_lines)]

mod network_utils;
mod utils;

mod bandwidth_utils;
use bandwidth_utils::{BandwidthCounter, TrafficMode, format_bandwidth};

mod modem_utils;
use modem_utils::{ModemError, ModemInfoParser, ModemStatus};

mod netgear_parser;
use netgear_parser::NetgearParser;

mod huawei_parser;
use huawei_parser::HuaweiParser;

mod bar_plot_widget;
mod res;

mod main_window;
use main_window::MainWindow;

use fltk::{app, prelude::*};
use std::thread;
use std::time::{Duration, SystemTime};

#[derive(Clone)]
enum Message {
    GetInfo,
    StartStopPolling,
    ReceivedInfo(Box<ModemStatus>),
    InfoOk,
    InfoError(ModemError),
    Quit,
}

const MANUFACTURERS: [&str; 2] = ["Netgear", "Huawei"];

const WIDTH: i32 = 840;
const HEIGHT: i32 = 435;

const DEFAULT_IP_ADDRESSES: [&str; 2] = ["192.168.1.1", "192.168.8.1"];

fn main() {
    let app = app::App::default();

    let (tx, rx) = app::channel::<Message>();
    let (info_thread_tx, info_thread_rx) = app::channel();

    let mut wnd = MainWindow::new(WIDTH, HEIGHT);

    /*
     * Initial state of UI
     */
    for m in MANUFACTURERS {
        wnd.model_choice.add_choice(m);
    }
    wnd.model_choice.set_value(0);

    // Fill list of standard IP addresses
    for ip_address in DEFAULT_IP_ADDRESSES {
        wnd.host_input.add(ip_address);
    }
    wnd.host_input.set_value(DEFAULT_IP_ADDRESSES[0]);

    wnd.connect_button.emit(tx, Message::StartStopPolling);
    wnd.close_button.emit(tx, Message::Quit);

    wnd.wnd.show();

    /*
     * Variables
     */
    let mut poller_timeout = Duration::from_secs(2);
    let mut run_poller = false;
    let mut host_address = String::new();
    let mut manufacturer_id: i32 = 0;

    let mut jh_getinfo: Option<std::thread::JoinHandle<()>> = None;

    let mut dlul = BandwidthCounter::new();

    /*
     * Run main event loop
     */
    {
        while app.wait() {
            if let Some(msg) = rx.recv() {
                match msg {
                    Message::StartStopPolling => {
                        run_poller = !run_poller;

                        if run_poller {
                            manufacturer_id = wnd.model_choice.value();

                            host_address = wnd.host_input.input().value();

                            let timeout = wnd.get_poll_timeout();
                            poller_timeout = Duration::from_secs(timeout);

                            tx.send(Message::GetInfo);
                            wnd.start_poll();
                        } else {
                            info_thread_tx.send(());
                            if let Some(jh) = jh_getinfo.take() {
                                if let Err(err) = jh.join() {
                                    eprintln!("Error: Thread Join: {err:?}");
                                }
                            }
                            wnd.stop_poll();
                        }
                    }
                    Message::GetInfo => {
                        let host_address = host_address.clone();
                        println!(
                            "Connecting to modem {} host {}",
                            MANUFACTURERS[manufacturer_id as usize], host_address
                        );

                        jh_getinfo = Some(thread::spawn(move || {
                            let modem_info = match manufacturer_id {
                                0 => NetgearParser::get_info(host_address.as_str()),
                                1 => HuaweiParser::get_info(host_address.as_str()),
                                _ => {
                                    eprintln!("Error: Unknown modem manufacturer ID");
                                    Err(ModemError::Unknown)
                                }
                            };

                            match modem_info {
                                Ok(modem_info) => {
                                    tx.send(Message::ReceivedInfo(Box::from(modem_info)));
                                    tx.send(Message::InfoOk);
                                }
                                Err(e) => {
                                    tx.send(Message::InfoError(e));
                                }
                            }

                            let start_time = SystemTime::now();

                            let mut still_running = true;
                            while still_running {
                                // Sleep for 100ms
                                thread::sleep(Duration::from_millis(100));

                                still_running = match info_thread_rx.recv() {
                                    Some(()) => {
                                        // Stop thread
                                        false
                                    }
                                    None => {
                                        // Run next poll
                                        true
                                    }
                                };

                                // Check if poller is still active
                                let current_time = SystemTime::now();
                                match current_time.duration_since(start_time) {
                                    Ok(t) => {
                                        if t >= poller_timeout {
                                            break;
                                        }
                                    }
                                    Err(err) => {
                                        eprintln!("Thread Timeout Error: {err:?}");
                                        break;
                                    }
                                }
                            }

                            if still_running {
                                // Run next poll
                                tx.send(Message::GetInfo);
                            }
                        }));
                    }
                    Message::ReceivedInfo(info) => {
                        println!("{info}\n");

                        wnd.set_info(&info);

                        if let Some(traffic_statistics) = info.traffic_statistics {
                            // Bandwidth
                            match info.traffic_mode {
                                TrafficMode::Absolute => {
                                    let dl_str = format_bandwidth(traffic_statistics.dl);
                                    let ul_str = format_bandwidth(traffic_statistics.ul);

                                    println!("Download : {dl_str} Upload : {ul_str}\n");

                                    wnd.set_bandwidth_data(traffic_statistics);
                                }
                                TrafficMode::Cumulative => {
                                    if let Some(dlul) =
                                        dlul.update_with_total_values(traffic_statistics)
                                    {
                                        let dl_str = format_bandwidth(dlul.dl);
                                        let ul_str = format_bandwidth(dlul.ul);

                                        println!("Download : {dl_str} Upload : {ul_str}\n");

                                        wnd.set_bandwidth_data(dlul);
                                    }
                                }
                            }
                        }
                    }
                    Message::InfoOk => {
                        wnd.set_error(None);
                    }
                    Message::InfoError(e) => match e {
                        ModemError::HttpConnection => wnd.set_error(Some("HTTP Error")),
                        ModemError::Access => wnd.set_error(Some("Access Error")),
                        ModemError::DataParsing => wnd.set_error(Some("Data Parsing Error")),
                        ModemError::Unknown => wnd.set_error(Some("Unknown error")),
                    },
                    Message::Quit => {
                        app.quit();
                    }
                }
            }
        }
    }

    // Stop threads
    {
        info_thread_tx.send(());
        if let Some(jh) = jh_getinfo.take() {
            if let Err(err) = jh.join() {
                eprintln!("Error: Thread Join: {err:?}");
            }
        }
    }
}
