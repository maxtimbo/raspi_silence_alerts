use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use tokio::sync::mpsc::{Sender, Receiver};

use crate::conf::MailConfig;
use raspi_silence_alerts::{format_duration, format_timestamp};

#[derive(Debug)]
pub struct PressInfo {
    pub started: SystemTime,
    pub last_alert: Option<SystemTime>,
    pub alerted: bool,
}

pub fn spawn_mail_thread(cfg: MailConfig, mut rx: Receiver<(String, String)>) {
    tokio::spawn(async move {
        while let Some((subject, body)) = rx.recv().await {
            if let Err(e) = cfg.send_mail(&subject, &body).await {
                eprintln!("failed to send mail: {}", e);
            }
        }
    });
}

pub fn spawn_alert_thread(
    press_times: Arc<Mutex<HashMap<u8, PressInfo>>>,
    pin_names: Arc<HashMap<u8, String>>,
    threshold: u64,
    alert_repeat: u64,
    mail_tx: Sender<(String, String)>,
) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;

            let now = SystemTime::now();
            let mut map = press_times.lock().unwrap();

            for (&pin, info) in map.iter_mut() {
                if let Ok(elapsed) = now.duration_since(info.started) {
                    if elapsed.as_secs() >= threshold {
                        let should_alert = match info.last_alert {
                            Some(last) => now.duration_since(last).map(|d| d.as_secs() >= alert_repeat).unwrap_or(false),
                            None => true,
                        };
                        if should_alert {
                            let name = pin_names.get(&pin).map(String::as_str).unwrap_or("unknown");
                            println!("send mail {} {} {:?}", name, pin, elapsed);
                            info.last_alert = Some(now);
                            info.alerted = true;
                            let subject = format!("{} SILENCE", name);
                            let body = format!("{} SILENCE for {}\nSILENCE BEGINS:\t{}", name, format_duration(elapsed), format_timestamp());
                            if let Err(e) = mail_tx.try_send((subject, body)) {
                                eprintln!("{}", e);
                            }
                        }
                    }
                }
            }
        }
    });
}
