use rumqttc::{MqttOptions, AsyncClient, QoS, Event, Incoming};
use std::time::Duration;
use std::env;
use std::str;
use std::error::Error;
use std::process::Command;


#[tokio::main(flavor="current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let mqtt_host = env::var("MQTT_HOST").or(Err("MQTT_HOST is required"))?;
    let mqtt_pass = env::var("MQTT_PASS").or(Err("MQTT_PASS is required"))?;
    let mqtt_user = env::var("MQTT_USER").or(Err("MQTT_USER is required"))?;
    let mut mqttoptions = MqttOptions::new("mqtt-projector-bridge", mqtt_host, 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(60));
    mqttoptions.set_credentials(mqtt_user, mqtt_pass);

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    client.subscribe("projector/power", QoS::AtMostOnce).await?;

    let device = "/dev/ttyUSB0";
    loop {
        let event = eventloop.poll().await?;
        let _ = match &event {
            Event::Incoming(Incoming::Publish(p)) => {
                let payload = str::from_utf8(&p.payload).unwrap();
                if p.topic == "projector/power" {
                    let cmd = match payload {
                        "on" => format!(r"echo -e '\x7e\x30\x30\x30\x30\x20\x31\x0d' > {device}"),
                        _ => format!(r"echo -e '\x7e\x30\x30\x30\x30\x20\x32\x0d' > {device}"),
                    };
                    Command::new("bash").args(["-c", &cmd]).output()?;
                }
            },
            _ => {}
        };
    }
}
