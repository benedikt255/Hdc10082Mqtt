use serde::{Deserialize, Serialize};


 #[derive(Serialize, Deserialize, Debug)]
 pub struct ConfigData {
   pub broker_url: String,
   pub broker_port: u16,
   pub broker_conn_timeout: u64,
   pub topic_temperature: String,
   pub topic_humidity: String,
   pub dev_hdc1008: String,
 }
