use clap::{Parser, Subcommand};
use telnet::{Event, Telnet};

/*****************************************************************************
 * Telnet connection helper
 *****************************************************************************/

const NETGEAR_TELNET_PORT: u16 = 5510;

struct ModemInfo {
    manufacturer: String,
    model: String,
    revision: String,
}

impl ModemInfo {
    fn default() -> Self {
        Self {
            manufacturer: String::default(),
            model: String::default(),
            revision: String::default(),
        }
    }
}

struct Connection {
    telnet: telnet::Telnet,
}

impl Connection {
    fn new(host: &str) -> Self {
        const BUF_SIZE: usize = 128;
        let telnet =
            Telnet::connect((host, NETGEAR_TELNET_PORT), BUF_SIZE).expect("Cannot connect to host");

        Self { telnet }
    }
    fn send(&mut self, cmd: &str) -> Option<Vec<String>> {
        let command_str = format!("{cmd}\r");
        let bytes = self
            .telnet
            .write(command_str.as_bytes())
            .inspect_err(|e| eprintln!("Telnet write error: {e:?}"))
            .ok()
            .unwrap_or_default();

        if bytes == 0 {
            eprintln!("Unable to write command to telnet");
            return None;
        }

        let mut str = String::default();
        loop {
            let event = self
                .telnet
                .read()
                .inspect_err(|e| eprintln!("Telnet read error: {e:?}"))
                .ok();

            match event {
                None => {
                    break;
                }
                Some(event) => match event {
                    Event::Data(buffer) => {
                        let s = String::from_utf8_lossy(&buffer);
                        str.push_str(&s);

                        if s.ends_with("\r\nOK\r\n") {
                            break;
                        }
                    }
                    Event::NoData => {
                        break;
                    }
                    _ => {
                        println!("Unhandled during reading from telnet: {event:?}");
                    }
                },
            }
        }

        Some(
            str.split("\r\n")
                .map(std::string::ToString::to_string)
                .collect(),
        )
    }
    fn ati(&mut self) -> Option<ModemInfo> {
        self.send("ATI").map(|lines| {
            let mut data = ModemInfo::default();

            for line in lines {
                if line.contains("Revision") {
                    data.revision = line
                        .split_once(':')
                        .unwrap_or_default()
                        .1
                        .trim()
                        .to_string();
                } else if line.contains("Model") {
                    data.model = line
                        .split_once(':')
                        .unwrap_or_default()
                        .1
                        .trim()
                        .to_string();
                } else if line.contains("Manufacturer") {
                    data.manufacturer = line
                        .split_once(':')
                        .unwrap_or_default()
                        .1
                        .trim()
                        .to_string();
                }
            }

            data
        })
    }
    fn gstatus(&mut self) {
        if let Some(lines) = self.send("AT !GSTATUS?") {
            println!("{:?}", &lines);
        }
    }
}

/*****************************************************************************
 * Argument parser
 *****************************************************************************/
#[derive(Parser)]
#[command(name = "netgear_telnet")]
#[command(bin_name = "netgear_telnet")]
#[command(about = "Test program for getting information from Netgear modem with telnet port", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: CliCommands,
}

#[derive(Debug, Subcommand)]
enum CliCommands {
    /// Info about modem
    ModemInfo { host: String },
    /// Connection status
    ConnectionStatus { host: String },
}

/*****************************************************************************
 * Commands implementation
 *****************************************************************************/
fn modem_info(host: &str) {
    let mut connection = Connection::new(host);
    let info = connection.ati();
    match info {
        None => {
            eprintln!("Cannot get ATI info");
        }
        Some(info) => {
            println!("Manufacturer: {}", info.manufacturer);
            println!("Model: {}", info.model);
            println!("Revision: {}", info.revision);
        }
    }
}

fn connection_status(host: &str) {
    let mut connection = Connection::new(host);
    connection.gstatus();
}

/*****************************************************************************
 * Main
 *****************************************************************************/

fn main() {
    let args = Cli::parse();

    match args.command {
        CliCommands::ModemInfo { host } => {
            modem_info(&host);
        }
        CliCommands::ConnectionStatus { host } => {
            connection_status(&host);
        }
    }
}
