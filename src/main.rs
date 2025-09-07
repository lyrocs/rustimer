use serialport;
use std::io::{Read, Write};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure serial port settings
    let port_name = "/dev/cu.usbserial-11230";
    // let baud_rate = 9600; // Common baud rate, adjust as needed
    let baud_rate = 115200; // Common baud rate, adjust as needed

    println!("Opening serial port: {}", port_name);

    // Open the serial port with longer timeout and additional settings
    let mut port = serialport::new(port_name, baud_rate)
        .dtr_on_open(true)
        .timeout(Duration::from_millis(5000)) // Increased to 5 seconds
        .data_bits(serialport::DataBits::Eight)
        .flow_control(serialport::FlowControl::None)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .open()?;

    println!("Serial port opened successfully");

    // Wait for Arduino to boot up after reset (triggered by opening serial connection)
    // println!("Waiting for Arduino to initialize...");
    // std::thread::sleep(Duration::from_millis(2000)); // 2 second delay

    // Clear any existing data in the input buffer
    let mut discard_buffer = [0u8; 256];
    while let Ok(bytes_read) = port.read(&mut discard_buffer) {
        if bytes_read == 0 {
            break;
        }
        println!("Cleared {} bytes from input buffer", bytes_read);
    }

    println!("Arduino should be ready now");

    // Data to send (0x3D)
    let data_to_send = [0x33];

    println!("Sending data: 0x{:02X}", data_to_send[0]);

    let mut response_buffer = [0u8; 256];
    let mut total_bytes_read = 0;

    // Send the data
    let output = "=";
    port.write(&data_to_send)?;
    port.flush()?; // Ensure data is actually sent

    match port.read(&mut response_buffer[total_bytes_read..]) {
        Ok(bytes_read) => {
            if bytes_read > 0 {
                total_bytes_read += bytes_read;
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
                // Check if there might be more data coming
                std::thread::sleep(Duration::from_millis(100));
            } else {
                println!("No data in this attempt");
            }
        }
        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
            println!("Timeout");
        }
        Err(e) => {
            println!("Error reading response: {}", e);
        }
    }

    println!("Closing serial port");
    Ok(())
}
