use colored::Colorize;
use serialport::*;
use std::{env, thread::sleep, time::Duration};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        eprintln!(
            "---\nUsage:\n{} <command> <port> <baudrate> <data(optional)>\n---",
            args[0]
        );
        return;
    }

    let command = &args[1];
    let port = &args[2];
    let baud_rate = match args[3].parse::<u32>() {
        Ok(rate) => rate,
        Err(_) => {
            eprintln!("Invalid baud rate specified");
            return;
        }
    };
    let port_name = if port == "auto" {
        auto_detect()
    } else {
        port.to_owned()
    };

    if validate_args(&port_name, baud_rate).is_err() {
        return;
    }

    match command.as_str() {
        "read" | "write" => {
            let data = if args.len() > 4 { Some(&args[4]) } else { None };
            execute_command(command, &port_name, baud_rate, data.map(|x| x.as_str()));
        }
        _ => {
            eprintln!(
                "Invalid command! Available commands: read, write\
                \n---\nUsage:\n{} <command> <port> <baudrate> <data(optional)>\n---",
                args[0]
            );
        }
    }
}

fn validate_args(port: &str, baud_rate: u32) -> Result<()> {
    if !available_ports()?.iter().any(|p| p.port_name == port) {
        return Err(serialport::Error::new(
            serialport::ErrorKind::InvalidInput,
            format!("Port {} not found", port),
        ));
    }

    if baud_rate == 0 {
        return Err(serialport::Error::new(
            serialport::ErrorKind::InvalidInput,
            "Invalid baud rate specified".to_string(),
        ));
    }

    Ok(())
}

// data structure from apps
//1F070960000000000000000000000000000000000000000062
fn execute_command(command: &str, port: &str, baud_rate: u32, data: Option<&str>) {
    let result: Result<()> = match command {
        "read" => read_data(port, baud_rate),
        "write" => {
            if let Some(data) = data {
                write_data(port, baud_rate, data)
            } else {
                eprintln!(
                    "To write, provide additional argument as data to send\
                    \n---\nUsage:\n{} write <port> <baudrate> <data>\n---",
                    env::args().next().unwrap()
                );
                return;
            }
        }
        _ => {
            eprintln!(
                "Invalid command! Available commands: read, write\
                \n---\nUsage:\n{} <command> <port> <baudrate> <data(optional)>\n---",
                env::args().next().unwrap()
            );
            return;
        }
    };

    if let Err(err) = result {
        eprintln!("Error: {}", err.to_string().red());
    }
}

fn read_data(port: &str, baud_rate: u32) -> Result<()> {
    serial_io(port, baud_rate, |port, serial_buf| {
        match port.read(serial_buf) {
            Ok(byte_read) => {
                if byte_read > 0 {
                    let data = String::from_utf8_lossy(&serial_buf[..byte_read]);
                    let cleaned_data = data.trim().replace(['\r', '\n'], "");
                    if !cleaned_data.is_empty() {
                        println!("Read {} bytes: {}", byte_read, cleaned_data);
                    }
                }
            }
            Err(_) => {
                return Err(serialport::Error::new(
                    serialport::ErrorKind::InvalidInput,
                    "No data found!".to_string(),
                ));
            }
        }
        Ok(())
    })
}

fn write_data(port: &str, baud_rate: u32, data: &str) -> Result<()> {
    serial_io(port, baud_rate, |port, _| {
        let rows = 8;

        for c in 0..rows {
            let complete_msg = if c == 0 {
                format!("w{}", data)
            } else {
                format!("a{}", data)
            };

            port.write_all(complete_msg.as_bytes())?;
            println!("Message '{}' written to serial port", complete_msg);

            sleep(Duration::from_millis(complete_msg.len() as u64));
        }
        Ok(())
    })
}

fn serial_io<F>(port: &str, baud_rate: u32, mut f: F) -> Result<()>
where
    F: FnMut(&mut Box<dyn SerialPort>, &mut [u8]) -> Result<()>,
{
    let mut port_ecu = serialport::new(port, baud_rate)
        .timeout(Duration::from_millis(100))
        .open()?;

    let mut serial_buf: Vec<u8> = vec![0; 15];

    loop {
        sleep(Duration::from_millis(100));
        f(&mut port_ecu, &mut serial_buf)?;
    }
}

fn auto_detect() -> String {
    let ports = available_ports().expect("No ports found");

    for p in &ports {
        println!("{}", p.port_name);
    }
    ports[0].port_name.clone()
}
