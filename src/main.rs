#![windows_subsystem = "windows"]

use chrono::Local;
use notification::notify;
mod notification;
mod regkey;
mod theme;

use {
    chrono::{Datelike, Duration},
    hotwatch::{Event, Hotwatch},
    serde::Deserialize,
    std::{
        env,
        fs::File,
        io::prelude::*,
        path::Path,
        sync::mpsc::{channel, RecvTimeoutError},
    },
    sunrise::sunrise_sunset,
    theme::{set_theme, ThemeVariant},
    win32_notification::NotificationBuilder,
};

#[derive(Deserialize, Debug)]
struct Location {
    latitude: f64,
    longitude: f64,
}

fn default_notifications() -> bool {
    true
}

fn default_refresh_period() -> u64 {
    60
}

#[derive(Deserialize, Debug)]
struct Config {
    #[serde(default)]
    disable: bool,
    #[serde(default = "default_notifications")]
    notifications: bool,
    #[serde(default)]
    invert: bool,
    #[serde(default = "default_refresh_period")]
    refresh_period: u64,
    location: Location,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ctrlc::set_handler(move || {
        std::process::exit(0);
    })?;

    let mut hotwatch = Hotwatch::new()?;
    let config_file_path = Path::new(&env::var("USERPROFILE")?).join(".autolight.toml");

    if !config_file_path.is_file() {
        notify("Error", "Couldn't find the configuration file. Exiting...");
        std::process::exit(1);
    }

    let (sender, receiver) = channel();

    hotwatch.watch(&config_file_path, move |event: Event| {
        if let Event::Write(_) = event {
            sender
                .send(())
                .expect("Failed to send signal to main thread.");
        }
    })?;

    loop {
        enum Action {
            ReloadConfig,
            Exit,
        }

        let config: Result<Config, _> = {
            let mut file = File::open(&config_file_path)?;
            let mut data = String::new();
            file.read_to_string(&mut data)?;

            toml::from_str(&data)
        };

        match config {
            Ok(config) if config.disable => {
                if config.notifications {
                    notify(
                        "autolight is disabled",
                        "Enable it in the configuration file",
                    );
                }

                std::process::exit(0);
            }
            Ok(config) => {
                match loop {
                    let now = Local::now();

                    dbg!(&now);

                    // sunrise & sunset are the unix timestamp (in seconds) of the sunrise/sunset
                    let (sunrise, sunset) = sunrise_sunset(
                        config.location.latitude,
                        config.location.longitude,
                        now.year(),
                        now.month(),
                        now.day(),
                    );

                    let now_timestamp = now.timestamp();

                    let (theme, wait_time) = if now_timestamp < sunrise {
                        (ThemeVariant::Dark, sunrise - now_timestamp)
                    } else if (sunrise..sunset).contains(&now_timestamp) {
                        (ThemeVariant::Light, sunset - now_timestamp)
                    } else {
                        // sunset < now_timestamp
                        // when will the sun rise tomorrow?
                        let tomorrow = now + Duration::days(1);

                        let (sunrise, _) = sunrise_sunset(
                            config.location.latitude,
                            config.location.longitude,
                            tomorrow.year(),
                            tomorrow.month(),
                            tomorrow.day(),
                        );

                        (ThemeVariant::Dark, sunrise - now_timestamp)
                    };

                    set_theme(if config.invert { theme.invert() } else { theme });

                    let wait_duration = std::time::Duration::from_secs(
                        (wait_time.max(0) as u64).min(config.refresh_period),
                    );
                    match receiver.recv_timeout(wait_duration) {
                        Ok(_) => break Action::ReloadConfig,
                        Err(RecvTimeoutError::Disconnected) => break Action::Exit,
                        _ => (),
                    }
                } {
                    Action::ReloadConfig => (),
                    Action::Exit => break,
                }
            }
            Err(err) => {
                NotificationBuilder::new()
                    .title_text("Invalid configuration")
                    .info_text(&err.to_string())
                    .build()
                    .unwrap()
                    .show()
                    .unwrap();

                if receiver.recv().is_err() {
                    break;
                }
            }
        }
    }

    Ok(())
}
