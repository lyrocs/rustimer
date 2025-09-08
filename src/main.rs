use serialport::{self, SerialPort};
use std::io::{Read, Write};
use std::time::Duration;

use serialport::{SerialPortType, available_ports};

fn list_ports() {
    match available_ports() {
        Ok(mut ports) => {
            // Let's output ports in a stable order to facilitate comparing the output from
            // different runs (on different platforms, with different features, ...).
            ports.sort_by_key(|i| i.port_name.clone());

            match ports.len() {
                0 => println!("No ports found."),
                1 => println!("Found 1 port:"),
                n => println!("Found {} ports:", n),
            };

            for p in ports {
                println!("    {}", p.port_name);
                match p.port_type {
                    SerialPortType::UsbPort(info) => {
                        println!("        Type: USB");
                        println!("        VID: {:04x}", info.vid);
                        println!("        PID: {:04x}", info.pid);
                        #[cfg(feature = "usbportinfo-interface")]
                        println!(
                            "        Interface: {}",
                            info.interface
                                .as_ref()
                                .map_or("".to_string(), |x| format!("{:02x}", *x))
                        );
                        println!(
                            "        Serial Number: {}",
                            info.serial_number.as_ref().map_or("", String::as_str)
                        );
                        println!(
                            "        Manufacturer: {}",
                            info.manufacturer.as_ref().map_or("", String::as_str)
                        );
                        println!(
                            "        Product: {}",
                            info.product.as_ref().map_or("", String::as_str)
                        );
                    }
                    SerialPortType::BluetoothPort => {
                        println!("        Type: Bluetooth");
                    }
                    SerialPortType::PciPort => {
                        println!("        Type: PCI");
                    }
                    SerialPortType::Unknown => {
                        println!("        Type: Unknown");
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("{:?}", e);
            eprintln!("Error listing serial ports");
        }
    }
}

// we receive bytes from the arduino that is a millis value in hex
// eg : 0x00 0x00 0x0D 0x59 0x66  should return 874854
fn convert_bytes_to_millis(bytes: [u8; 256]) -> u32 {
    let mut millis = 0;
    millis = (bytes[0] as u32) << 24;
    millis += (bytes[1] as u32) << 16;
    millis += (bytes[2] as u32) << 8;
    millis += bytes[3] as u32;
    millis
}

fn open_port(
    port_name: &str,
    baud_rate: u32,
) -> Result<Box<dyn SerialPort>, Box<dyn std::error::Error>> {
    let mut port = serialport::new(port_name, baud_rate)
        .dtr_on_open(true)
        .timeout(Duration::from_millis(5000)) // Increased to 5 seconds
        .data_bits(serialport::DataBits::Eight)
        .flow_control(serialport::FlowControl::None)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .open()?;

    let mut discard_buffer = [0u8; 256];
    while let Ok(bytes_read) = port.read(&mut discard_buffer) {
        if bytes_read == 0 {
            break;
        }
        println!("Cleared {} bytes from input buffer", bytes_read);
    }

    println!("Arduino should be ready now");
    Ok(port)
}

fn send_and_read(
    port: &mut Box<dyn SerialPort>,
    data_to_send: &[u8],
    debug: bool,
) -> Result<[u8; 256], Box<dyn std::error::Error>> {
    let mut response_buffer: [u8; 256] = [0u8; 256];
    let mut total_bytes_read = 0;
    port.write(&data_to_send)?;
    port.flush()?; // Ensure data is actually sent

    match port.read(&mut response_buffer[total_bytes_read..]) {
        Ok(bytes_read) => {
            if bytes_read > 0 {
                total_bytes_read += bytes_read;

                if debug {
                    println!("Received {} bytes in this attempt", bytes_read);

                    print!("Hex: ");
                    for i in 0..total_bytes_read {
                        print!("0x{:02X} ", response_buffer[i]);
                    }
                    println!();

                    print!("ASCII: ");
                    for i in 0..total_bytes_read {
                        let byte = response_buffer[i];
                        if byte >= 32 && byte <= 126 {
                            print!("{}", byte as char);
                        } else {
                            print!(".");
                        }
                    }
                    println!();
                }
                // Check if there might be more data coming
                // std::thread::sleep(Duration::from_millis(100));
                return Ok(response_buffer);
            } else {
                println!("No data in this attempt");
                return Err("No data received".into());
            }
        }
        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
            println!("Timeout");
            return Err("Read timeout".into());
        }
        Err(e) => {
            println!("Error reading response: {}", e);
            return Err(e.into());
        }
    }
}

fn read_ms(port: &mut Box<dyn SerialPort>) -> Result<u32, Box<dyn std::error::Error>> {
    let data_to_send = [0x33];
    let response_buffer = send_and_read(port, &data_to_send, false)?;
    let millis = convert_bytes_to_millis(response_buffer);
    Ok(millis)
}

fn read_version(port: &mut Box<dyn SerialPort>) -> Result<u32, Box<dyn std::error::Error>> {
    let data_to_send = [0x3D];
    let response_buffer = send_and_read(port, &data_to_send, true)?;
    let version = response_buffer[1];
    Ok(version.into())
}

fn read_peak(port: &mut Box<dyn SerialPort>) -> Result<u32, Box<dyn std::error::Error>> {
    let data_to_send = [0x0D]; // 0E
    let response_buffer = send_and_read(port, &data_to_send, false)?;

    let lap_id = response_buffer[0];
    // ms_val is a 16 bit value
    let ms_val = (response_buffer[1] as u16 * 256 + response_buffer[2] as u16) as u32;
    let rssi = response_buffer[3];

    println!("lap_id: {}, ms_val: {}, rssi: {}", lap_id, ms_val, rssi);
    Ok(rssi as u32)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // list_ports();

    let port_name = "/dev/cu.usbserial-11230";
    let baud_rate = 115200;
    let mut port = open_port(port_name, baud_rate)?;

    let millis = read_ms(&mut port)?;
    println!("ms value: {}", millis);

    // let version = read_version(&mut port)?;
    // println!("version value: {}", version);

    for _ in 0..2 {
        let peak = read_peak(&mut port)?;
        // println!("peak value: {}", peak);
        std::thread::sleep(Duration::from_millis(1000));
    }

    let millis = read_ms(&mut port)?;
    println!("ms value: {}", millis);

    Ok(())
}
