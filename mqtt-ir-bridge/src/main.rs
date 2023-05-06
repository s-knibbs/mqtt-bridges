use rumqttc::{MqttOptions, AsyncClient, QoS};
use evdev::{Device, InputEventKind};
use tokio::task;
use std::time::Duration;
use std::env;
use std::error::Error;


#[tokio::main(flavor="current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let mqtt_host = env::var("MQTT_HOST").or(Err("MQTT_HOST is required"))?;
    let mqtt_pass = env::var("MQTT_PASS").or(Err("MQTT_PASS is required"))?;
    let mqtt_user = env::var("MQTT_USER").or(Err("MQTT_USER is required"))?;
    let device_path = env::var("DEVICE_PATH").or(Err("DEVICE_PATH is required"))?;
    let mut mqttoptions = MqttOptions::new("mqtt-ir-bridge", mqtt_host, 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(60));
    mqttoptions.set_credentials(mqtt_user, mqtt_pass);

    let device = Device::open(device_path)?;
    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    let mut events = device.into_event_stream()?;

    task::spawn(async move {
        loop {
            eventloop.poll().await.unwrap();
        }
    });

    loop {
        let ev = events.next_event().await?;
        let kind = ev.kind();
        if ev.value() == 1 {
            let key_str = match &kind {
                InputEventKind::Key(key) => format!("{:?}", key),
                _ => continue,
            };
            client.publish("remote/key_press", QoS::AtLeastOnce, false, key_str.as_bytes()).await?;
        }
    }
}
