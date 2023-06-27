use colored::Colorize;
use serialport::*;
use std::{
    env,
    thread::sleep,
    time::{Duration, Instant},
};

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

    match command.as_str() {
        "read" => read_data(&port_name, baud_rate),
        "write" => {
            if args.len() > 4 {
                let data = &args[4];
                write_data(&port_name, baud_rate, data);
            } else {
                eprintln!(
                    "To write, provide additional argument as data to send\
                    \n---\nUsage:\n{} write <port> <baudrate> <data>\n---",
                    args[0]
                );
            }
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
//1F070960000000000000000000000000000000000000000062
fn read_data(port: &str, baud_rate: u32) {
    if let Err(err) = read_port(port, baud_rate, 15) {
        eprintln!("Error: {}", err.to_string().red());
    } else {
        println!("Read successful");
    }
}

fn write_data(port: &str, baud_rate: u32, data: &str) {
    if let Err(err) = write_to_port(port, baud_rate, data) {
        eprintln!("Error: {}", err.to_string().red());
    } else {
        println!("Write successful");
    }
}

fn auto_detect() -> String {
    let ports = available_ports().expect("No ports found");

    for p in &ports {
        println!("{}", p.port_name);
    }
    ports[0].port_name.clone()
}

fn read_port(port: &str, baud_rate: u32, num_bytes: usize) -> Result<()> {
    let mut port_ecu = serialport::new(port, baud_rate)
        .timeout(Duration::from_millis(100))
        .open()?;

    let mut serial_buf: Vec<u8> = vec![0; num_bytes];
    let error_msg = "No data found!";

    loop {
        sleep(Duration::from_millis(100));
        match port_ecu.read(serial_buf.as_mut_slice()) {
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
                eprintln!("Failed: {}", error_msg.red());
                break;
            }
        }
    }
    Ok(())
}

fn write_to_port(port: &str, baud_rate: u32, message: &str) -> Result<()> {
    let start = Instant::now();
    let mut port_ecu = serialport::new(port, baud_rate)
        .timeout(Duration::from_millis(100))
        .open()?;

    let rows = 8;

    for c in 0..rows {
        let complete_msg = if c == 0 {
            format!("w{}", message)
        } else {
            format!("a{}", message)
        };

        port_ecu.write_all(complete_msg.as_bytes())?;
        println!("Message '{}' written to serial port", complete_msg);

        sleep(Duration::from_millis(complete_msg.len() as u64));
    }
    let duration = start.elapsed();
    println!("Elapsed time: {:?}", duration);

    Ok(())
}
