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
    let speaker_mac = env::var("SPEAKER_MAC").or(Err("SPEAKER_MAC is required"))?;
    let input_mac = env::var("INPUT_MAC").or(Err("INPUT_MAC, is required"))?;
    let mut mqttoptions = MqttOptions::new("mqtt-ue-boom-bridge", mqtt_host, 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(60));
    mqttoptions.set_credentials(mqtt_user, mqtt_pass);

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    client.subscribe("bt_speaker/power", QoS::AtMostOnce).await?;

    loop {
        let event = eventloop.poll().await?;
        let _ = match &event {
            Event::Incoming(Incoming::Publish(p)) => {
                let payload = str::from_utf8(&p.payload).unwrap();
                if p.topic == "bt_speaker/power" {
                    let cmd = match payload {
                        "on" => format!("{input_mac}01"),
                        "off" => format!("{input_mac}02"),
                        _ => format!("{input_mac}02")
                    };
                    let cmd_result = Command::new("gatttool")
                        .args(["-i", "hci0", "-b", &speaker_mac, "--char-write-req", "-a", "0x0003", "-n", &cmd])
                        .output();
                    match &cmd_result {
                        Err(e) => println!("Command failed: {e:?}"),
                        Ok(output) => {
                            if output.status.code() != Some(0) {
                                println!("Command failed: {}", str::from_utf8(&output.stderr).unwrap());
                            }
                        }
                    }
                }
            },
            _ => {}
        };
    }
}
