mod conf;
mod send_mail;
mod alerts;

use anyhow::Result;
use rppal::gpio::{Event, Gpio, Trigger};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use std::thread;

use alerts::{spawn_alert_thread, spawn_mail_thread, PressInfo};
use raspi_silence_alerts::{format_duration, format_timestamp};
use conf::Config;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::read("/etc/silence_sense.json")?;
    let mail_cfg = config.mail_config();
    let alert_repeat: u64 = config.alert_repeat * 60;

    let pin_names: HashMap<u8, String> = config.pins
        .iter()
        .map(|(name, pin_str)| (pin_str.parse::<u8>().unwrap(), name.clone()))
        .collect();
    let pin_names = Arc::new(pin_names);

    let gpio = Gpio::new()?;
    let press_times = Arc::new(Mutex::new(HashMap::<u8, PressInfo>::new()));
    let mut pins = Vec::new();

    let (mail_tx, mail_rx) = tokio::sync::mpsc::channel::<(String, String)>(100);
    spawn_mail_thread(mail_cfg.clone(), mail_rx);

    spawn_alert_thread(
        press_times.clone(),
        pin_names.clone(),
        config.threshold,
        alert_repeat,
        mail_tx.clone(),
    );

    for (name, pin_str) in &config.pins {
        let mail_tx = mail_tx.clone();
        let pin_num: u8 = match pin_str.parse() {
            Ok(p) => p,
            Err(_) => {
                eprintln!("Invalid pin number {} for {}", pin_str, name);
                continue;
            }
        };
        let press_times = press_times.clone();
        let name = name.clone();

        {
            let pin = gpio.get(pin_num)?.into_input_pullup();
            if pin.is_low() {
                let now = SystemTime::now();
                println!("{} pin {} initially low at {}", name, pin_num, format_timestamp());
                press_times.lock().unwrap().insert(
                    pin_num,
                    PressInfo {
                        started: now,
                        last_alert: None,
                        alerted: false,
                    }
                );
            }
        }

        let mut pin = gpio.get(pin_num)?.into_input_pullup();

        let press_times = press_times.clone();
        let name = name.clone();

        pin.set_async_interrupt(
            Trigger::Both,
            Some(Duration::from_millis(50)),
            move |event: Event| {
                let now = SystemTime::now();
                match event.trigger {
                    Trigger::FallingEdge => {
                        println!("{} pin {} pressed at {}", name, pin_num, format_timestamp());
                        press_times.lock().unwrap().insert(
                            pin_num,
                            PressInfo {
                                started: now,
                                last_alert: None,
                                alerted: false,
                            },
                        );
                    }
                    Trigger::RisingEdge => {
                        let mut map = press_times.lock().unwrap();
                        if let Some(info) = map.remove(&pin_num) {
                            if let Ok(held_for) = now.duration_since(info.started) {
                                println!("{} pin {} released at {}, held for {}",
                                    name, pin_num, format_timestamp(), format_duration(held_for));
                                if info.alerted {
                                    if let Err(e) = mail_tx.try_send((
                                            format!("{} ENDED SILENCE", name),
                                            format!(
                                                "{} ended silence. Silent for {} - ended at {}",
                                                name,
                                                format_duration(held_for),
                                                format_timestamp()
                                                )
                                            )
                                        ) 
                                    {
                                        eprintln!("Failed: {}", e);
                                    } else {
                                        println!("sent email NAME{} PIN{} HELD_FOR{}", name, pin_num, format_duration(held_for));
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        )?;
        pins.push(pin);
    }
    println!("Listening for pins");

    loop {
        thread::sleep(Duration::from_secs(60));
    }
}
