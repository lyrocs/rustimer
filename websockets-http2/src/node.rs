use serialport::{self, SerialPort};
use std::time::Duration;

pub fn open_port(
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

pub fn send_and_read(
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

pub fn read_peak(port: &mut Box<dyn SerialPort>) -> u32 {
    let data_to_send = [0x0D]; // 0E
    match send_and_read(port, &data_to_send, false) {
        Ok(response_buffer) => {
            let _lap_id = response_buffer[0];
            // ms_val is a 16 bit value
            let _ms_val = (response_buffer[1] as u16 * 256 + response_buffer[2] as u16) as u32;
            let rssi = response_buffer[3];

            // println!("lap_id: {}, ms_val: {}, rssi: {}", lap_id, ms_val, rssi);
            rssi as u32
        }
        Err(_) => 0,
    }
}
