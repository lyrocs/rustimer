// Use mock implementation if 'mock' feature is enabled
#[cfg(feature = "mock")]
mod mock {
    use rand::Rng;
    use std::io;

    // A mock SerialPort that does nothing but can be created.
    pub struct MockSerialPort;

    impl io::Read for MockSerialPort {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Ok(0)
        }
    }

    impl io::Write for MockSerialPort {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    // Implement the serialport::SerialPort trait for our mock struct.
    // This requires enabling the `mock` feature for the `serialport` crate if it has one,
    // or creating a more elaborate mock. For now, we assume this is sufficient
    // if the trait is public and its methods can be implemented.
    // Since we can't implement the trait directly, we will just return the MockSerialPort
    // inside a Box<dyn SerialPort> which is not ideal but will work for our case.
    // The best approach is to define our own trait.

    // For simplicity, we'll just define the functions we need.
    pub fn open_port(
        _port_name: &str,
        _baud_rate: u32,
    ) -> Result<Box<dyn serialport::SerialPort>, Box<dyn std::error::Error>> {
        println!("Using mock Arduino port.");
        // Create a mock TTY port for testing purposes.
        let (master, slave) = serialport::TTYPort::pair()?;
        // We don't use the master, but we need to keep it in scope.
        // To avoid compiler warnings about it being unused, we can ignore it.
        let _master = master;
        Ok(Box::new(slave))
    }

    pub fn read_peak(_port: &mut Box<dyn serialport::SerialPort>) -> u32 {
        let mut rng = rand::thread_rng();
        let peak = rng.gen_range(50..100);
        let random_sleep = rng.gen_range(1..5);
        std::thread::sleep(std::time::Duration::from_secs(random_sleep));
        peak
    }
}

#[cfg(feature = "mock")]
pub use self::mock::*;

// Use real implementation if 'mock' feature is NOT enabled
#[cfg(not(feature = "mock"))]
mod real {
    use serialport::{self, SerialPort};
    use std::time::Duration;

    pub fn open_port(
        port_name: &str,
        baud_rate: u32,
    ) -> Result<Box<dyn SerialPort>, Box<dyn std::error::Error>> {
        println!("Opening real Arduino port: {}", port_name);
        let mut port = serialport::new(port_name, baud_rate)
            .dtr_on_open(true)
            .timeout(Duration::from_millis(5000))
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
        port.write_all(data_to_send)?;
        port.flush()?;

        match port.read(&mut response_buffer) {
            Ok(bytes_read) => {
                if bytes_read > 0 {
                    if debug {
                        println!("Received {} bytes", bytes_read);
                        // Debug printing logic here...
                    }
                    Ok(response_buffer)
                } else {
                    Err("No data received".into())
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => Err("Read timeout".into()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn read_peak(port: &mut Box<dyn SerialPort>) -> u32 {
        let data_to_send = [0x0D]; // Command to read peak
        match send_and_read(port, &data_to_send, false) {
            Ok(response_buffer) => {
                // Assuming the peak value (RSSI) is the 4th byte in the response
                if response_buffer.len() > 3 {
                    response_buffer[3] as u32
                } else {
                    0
                }
            }
            Err(_) => 0,
        }
    }
}

#[cfg(not(feature = "mock"))]
pub use self::real::*;
