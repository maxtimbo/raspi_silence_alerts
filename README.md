# Raspberry Pi Silence Alert System

A small Rust daemon that monitors GPIO press events and sends email alerts when a threshold is reached and repeats the alert after [n] minutes.

This is designed to be installed on a Raspberry Pi that is connected to a commercial silence sensor.

## Features
- Sends email alerts
- configurable pins and names
- configurable threshold and repeating alerts

## Build instructions
Don't try to build this on a Raspberry Pi. When I tried to do this, it took forever and ultimately failed because the pi froze.

I had very good success using [cross-rs](https://github.com/cross-rs/cross) and then:

```
$ cross build --release --target aarch64-unknown-linux-gnu
```

Once that's built, you can upload the bin and config file to your pi:

```
$ sftp pi@your_pi.local
Connected to your_pi.local
sftp> put target/aarch64-unknown-linux-gnu/release/raspi_silence_alerts
sftp> put silence_sense.json
```

SSH into your pi and edit the `silence_sense.json` file. Copy or move it to `/etc/silence_sense.json`. Also, copy or move `raspi_silence_alerts` to `/usr/local/bin/raspi_silence_alerts`:

```
$ cp raspi_silence_alerts /usr/local/bin/
$ cp silence_sense.json /etc/
```

Once you have the bin and the config file in place, you can create a service for `raspi_silence_alerts`:

```
$ sudo vim /etc/systemd/system/silence-sensor.service
```

```
[Unit]
Description=Silence Sensor GPIO Watcher Email Alerts
After=network.target

[Service]
ExecStart=/usr/local/bin/raspi_silence_alerts
Restart=on-failure
StandardOutput=append:/var/log/silence-sensor.log
StandardError=append:/var/log/silence-sensor.log

[Install]
WantedBy=multi-user.target
```

Reload systemd daemon, start the service, check the status for errors, enable the service, and finally check the logs:

```
$ sudo systemctl daemon-reload
$ sudo systemctl start silence-sensor.service
$ sudo systemctl status silence-sensor.service
$ sudo systemctl enable silence-sensor.service
$ sudo tail -f /var/log/silence-sensor.log
```
