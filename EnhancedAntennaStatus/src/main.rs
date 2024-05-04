mod bandwidth_utils;
use bandwidth_utils::*;

mod modem_utils;
use modem_utils::*;

mod utils;

mod res;
mod bar_plot_widget;

mod main_window;
use main_window::*;

use fltk::{*, prelude::*};
use std::thread;
use std::time::{Duration, SystemTime};

// const POLLER_TIMEOUT: u64 = 2_000;

#[derive(Clone, Copy)]
enum Message {
    GetInfo,
    StartStopPolling,
    ReceivedInfo(ModemStatus),
    InfoError(i32),
    Quit,
}

fn main() {
    let app = app::App::default();

    let (tx, rx) = app::channel::<Message>();
    let (info_thread_tx, info_thread_rx) = app::channel();

    const WIDTH: i32 = 840;
    const HEIGHT: i32 = 435;
    let mut wnd = MainWindow::new(WIDTH, HEIGHT);

    /*
     * Initial state of UI
     */
    wnd.host_input.set_value("192.168.1.1");
    wnd.connect_button.emit(tx, Message::StartStopPolling);
    wnd.close_button.emit(tx, Message::Quit);

    wnd.wnd.show();

    /*
     * Variables
     */
    let mut poller_timeout = Duration::from_secs(2);
    let mut run_poller = false;
    let mut host_address = "".to_string();

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
                            host_address = wnd.host_input.value();

                            let timeout = wnd.get_poll_timeout();
                            poller_timeout = Duration::from_secs(timeout);

                            tx.send(Message::GetInfo);
                            wnd.start_poll();
                        }
                        else {
                            info_thread_tx.send(());
                            if let Some(jh) = jh_getinfo.take() {
                                if let Err(err) = jh.join() {
                                    eprintln!("Error: Thread Join: {err:?}");
                                }
                            }
                            wnd.stop_poll();
                        }
                    },
                    Message::GetInfo => {
                        let host_address = host_address.clone();
                        println!("Connecting to host {}", host_address);
                        
                        jh_getinfo = Some(thread::spawn(
                            move || {
                                if let Some(modem_info) = NetgearParser::get_info(host_address.as_str()) {
                                    tx.send(Message::ReceivedInfo(modem_info));
                                    tx.send(Message::InfoError(0));
                                }
                                else {
                                    tx.send(Message::InfoError(-1));
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
                                        },
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
                            }
                        ));
                    },
                    Message::ReceivedInfo(info) => {
                        println!("{info}\n");

                        wnd.set_info(info);

                        // Bandwidth
                        if let Some(dlul) = dlul.update_with_total_values((info.dl, info.ul)) {
                            let dl_str = format_bandwidth(dlul.0);
                            let ul_str = format_bandwidth(dlul.1);

                            println!("Download : {dl_str} Upload : {ul_str}\n");

                            wnd.set_bandwidth_data(dlul);
                        }
                    },
                    Message::Quit => {
                        app.quit();
                    },
                    Message::InfoError(i) => {
                        match i {
                            -1 => wnd.set_error("HTTP Error"),
                            _ => wnd.set_error(""),
                        }
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
