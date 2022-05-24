extern crate i2cdev;
extern crate paho_mqtt as mqtt;

use std::env;
use std::process;
use std::thread;
use std::time::Duration;

use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};

const NUNCHUCK_SLAVE_ADDR: u16 = 0x40;
const DFLT_BROKER:&str = "tcp://172.25.130.227:1883";
const DFLT_CLIENT:&str = "local_tempHumidity";
const DFLT_TOPICS_HUMID:&str = "home/livingroom/humidity";
const DFLT_TOPICS_TEMP:&str = "home/livingroom/temperature";
const QOS:i32 = 0;
const SLEEPTIME:u64 = 60;

fn main() {
    i2cfun();
}

fn i2cfun() -> Result<(), LinuxI2CError> {
    let mut dev = LinuxI2CDevice::new("/dev/i2c-1", NUNCHUCK_SLAVE_ADDR)?;

    // init sequence
    dev.smbus_write_word_data(0x02, 0x0090)?;
    
    let host = env::args().nth(1).unwrap_or_else(||
        DFLT_BROKER.to_string()
    );

    // Define the set of options for the create.
    // Use an ID for a persistent session.
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(host)
        .client_id(DFLT_CLIENT.to_string())
        .finalize();

    // Create a client.
    let cli = mqtt::Client::new(create_opts).unwrap_or_else(|err| {
        println!("Error creating the client: {:?}", err);
        process::exit(1);
    });

    // Define the set of options for the connection.
    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(SLEEPTIME*2))
        .clean_session(true)
        .finalize();

    // Connect and wait for it to complete or fail.
    if let Err(e) = cli.connect(conn_opts) {
        println!("Unable to connect:\n\t{:?}", e);
        process::exit(1);
    }

    loop {
        let mut buf: [u8; 4] = [0; 4];
        dev.smbus_write_byte(0x00).unwrap();
        thread::sleep(Duration::from_millis(20));
        dev.read(&mut buf).unwrap();
        println!("Reading: {:?}", buf);
        let temp = ((buf[0] as u16 * 256 + buf[1] as u16) as f64)/65536.0 * 165.0 - 40.0;
        let humid = ((buf[2] as u16 * 256 + buf[3] as u16) as f64)/65536.0 * 100.0;
        println!("temp {:?} °C humid {:?} %", temp, humid);
        {
            let msg = mqtt::Message::new(DFLT_TOPICS_HUMID, humid.to_string(), QOS);
            let tok = cli.publish(msg);

            if let Err(e) = tok {
                println!("Error sending message: {:?}", e);
                }
        }
        {
            let msg = mqtt::Message::new(DFLT_TOPICS_TEMP, temp.to_string(), QOS);
            let tok = cli.publish(msg);

            if let Err(e) = tok {
                println!("Error sending message: {:?}", e);
                }
        }
        thread::sleep(Duration::from_secs(SLEEPTIME));
    }


    // Disconnect from the broker.
    let tok = cli.disconnect(None);
    println!("Disconnect from the broker");
    tok.unwrap();
}
