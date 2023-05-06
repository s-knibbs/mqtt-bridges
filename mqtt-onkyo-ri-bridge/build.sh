#!/bin/bash

g++ -o onkyo-mqtt-bridge --std=c++11 *.cpp -lpaho-mqttpp3 -lpaho-mqtt3as -lwiringPi
