#include <iostream>
#include <cstdlib>
#include <string>
#include <cstring>
#include <cctype>
#include <thread>
#include <chrono>
#include "mqtt/client.h"

#include "OnkyoRI.h"

#define CMD_ON 0x02f
#define CMD_OFF 0x0da
#define CMD_SRC_DOCK 0x170
#define CMD_SRC_CD 0x020
#define CMD_SRC_NEXT 0x0d5
#define CMD_SRC_PREV 0x0d6
#define CMD_VOL_UP 0x002
#define CMD_VOL_DOWN 0x003

using namespace std;
using namespace std::chrono;

const string CLIENT_ID      { "onkyo_ri_mqtt_bridge" };

/////////////////////////////////////////////////////////////////////////////

void message_loop(mqtt::client& cli, OnkyoRI& onk) {
    // Consume messages

    while (true) {
        auto msg = cli.consume_message();

        if (msg) {
            auto topic = msg->get_topic();
            auto payload = msg->to_string();
            if (topic == "switch/led") {
                if (payload == "ON") {
                    system("echo 1 | sudo tee /sys/class/leds/led1/brightness");
                } else if (payload == "OFF") {
                    system("echo 0 | sudo tee /sys/class/leds/led1/brightness");
                }
            } else if (topic == "amp/power") {
                if (payload == "ON") {
                    onk.send(CMD_ON);
                } else if (payload == "OFF") {
                    onk.send(CMD_OFF);
                }
            } else if (topic == "amp/volume") {
                if (payload == "DOWN") {
                    onk.send(CMD_VOL_DOWN);
                } else if (payload == "UP") {
                    onk.send(CMD_VOL_UP);
                }
            } else if (topic == "amp/source") {
                if (payload == "CD") {
                    onk.send(CMD_SRC_CD);
                } else if (payload == "DOCK") {
                    onk.send(CMD_SRC_DOCK);
                } else if (payload == "NEXT") {
                    onk.send(CMD_SRC_NEXT);
                } else if (payload == "PREV") {
                    onk.send(CMD_SRC_PREV);
                }
            } else {
                cout << msg->get_topic() << ": " << msg->to_string() << endl;
            }
        } else if (!cli.is_connected()) {
            cout << "Lost connection" << endl;
            return;
        }
    }
}

int main(int argc, char* argv[])
{
    char* server_address = getenv("MQTT_HOST");
    if (server_address == NULL) {
        cerr << "Missing 'MQTT_HOST' env var" << endl;
        return 1;
    }
    mqtt::client cli(server_address, CLIENT_ID);

    char* user = getenv("MQTT_USER");
    char* pass = getenv("MQTT_PASS");

    if (user ==  NULL || pass == NULL) {
        cerr << "Missing 'MQTT_USER' and/or 'MQTT_PASS' env vars" << endl;
        return 1;
    }

    auto connOpts = mqtt::connect_options_builder()
        .user_name(user)
        .password(pass)
        .keep_alive_interval(seconds(60))
        .clean_session(true)
        .finalize();

    const vector<string> TOPICS {
        "switch/led",
        "amp/power",
        "amp/volume",
        "amp/source",
    };
    const vector<int> QOS { 0, 0, 0, 0 };

    int gpio = 0;
    char* pin = getenv("GPIO_PIN");
    if (pin != NULL) {
        gpio = (int) strtol(pin, NULL, 10);
    }
    OnkyoRI onk = OnkyoRI(gpio);
    int connection_tries = 4;

    while(true) {
        cout << "Connecting to the MQTT server..." << flush;
        try {
            mqtt::connect_response rsp = cli.connect(connOpts);
            if (!rsp.is_session_present()) {
                std::cout << "Subscribing to topics..." << std::flush;
                cli.subscribe(TOPICS, QOS);
                std::cout << "OK" << std::endl;
            }
            else {
                cout << "Session already present. Skipping subscribe." << std::endl;
            }
            cout << "OK\n" << endl;
            break;
        } catch (const mqtt::exception& exc) {
            cerr << exc.what() << endl;
            if (connection_tries == 0) {
                return 2;
            }
            this_thread::sleep_for(milliseconds(1000));
            connection_tries--;
        } 
    }

    try {
        message_loop(cli, onk);
        if (cli.is_connected()) {
            // Disconnect
            cout << "\nDisconnecting from the MQTT server..." << flush;
            cli.disconnect();
            cout << "OK" << endl;
        } else {
            return 1;
        }
    }
    catch (const mqtt::exception& exc) {
        cerr << exc.what() << endl;
        return 2;
    }

    return 0;
}

