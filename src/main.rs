extern crate i2cdev;
pub mod config;

use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};
use rumqttc::{MqttOptions, Client, QoS};
use serde_json;
use std::fs::File;
use std::io::BufReader;
use std::thread;
use std::time::Duration;


use crate::config::ConfigData;

const NUNCHUCK_SLAVE_ADDR: u16 = 0x40;
const DFLT_CLIENT:&str = "local_tempHumidity";
const QOS:QoS = QoS::AtMostOnce;
const SLEEPTIME:u64 = 60;

fn main() {
    i2cfun();
}

fn i2cfun() -> Result<(), LinuxI2CError> {
    let file = File::open("config/Hdc10082Mqtt.json")?;
    let reader = BufReader::new(file);
    let config: ConfigData = serde_json::from_reader(reader).expect("error while reading or parsing");
    println!("config read, open i2c-dev {:?}", config.dev_hdc1008);
    let mut dev = LinuxI2CDevice::new(config.dev_hdc1008, NUNCHUCK_SLAVE_ADDR)?;

    // init sequence
    dev.smbus_write_word_data(0x02, 0x0090)?;

    println!("sensor initialised, connect to MQTT broker");
    //create mqtt client and connect
    let mut mqttoptions = MqttOptions::new(DFLT_CLIENT.to_string(), config.broker_url, config.broker_port);
    mqttoptions.set_keep_alive(Duration::from_secs(config.broker_conn_timeout));

    let (mut client, mut connection) = Client::new(mqttoptions, 10);

    thread::spawn(move || for (_i, notification) in connection.iter().enumerate() {
        println!("Notification = {:?}", notification);
        thread::sleep(Duration::from_millis(1000));
    });

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
            let tok = client.publish(config.topic_humidity.as_str(), QOS, true, humid.to_string());

            if let Err(e) = tok {
                println!("Error sending message: {:?}", e);
                }
        }
        {
            let tok = client.publish(config.topic_temperature.as_str(), QOS, true, temp.to_string());

            if let Err(e) = tok {
                println!("Error sending message: {:?}", e);
                }
        }
        thread::sleep(Duration::from_secs(SLEEPTIME));
    }
}
