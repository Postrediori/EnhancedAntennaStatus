use telnet::{Telnet, Event};

fn get_modem_at_status(host: &str) -> Option<String> {
    let mut telnet = Telnet::connect((host, 5510), 128)
        .expect("Couldn't connect to the server...");

    let command_str = "at !gstatus?\r\n";
    telnet.write(command_str.as_bytes()).expect("Read error");

    let mut str: String = "".to_string();
    loop {
        let event = telnet.read().expect("Read error");

        match event {
            Event::Data(buffer) => {
                let s = String::from_utf8_lossy(&buffer);
                // println!("{:?}", s);
                str.push_str(&s);

                if s.ends_with("\r\nOK\r\n") {
                    break;
                }
            },
            Event::NoData => { break; },
            _ => { println!("{:?}", event); }
        }
    }

    Some(str)
}

fn main() {
    let host = "192.168.1.1";

    if let Some(str) = get_modem_at_status(host) {
        print!("{}", str);
    }
    else {
        eprintln!("Error: Cannot get modem AT status\n");
    }
}
