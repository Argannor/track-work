use std::{error::Error, time::Duration};

use argh::FromArgs;
use lazy_static::lazy_static;
use std::sync::RwLock;
use config::Config;

use crate::crossterm::run;
use crate::app_config::AppConfig;

mod app;
mod input;
mod widgets;
mod crossterm;
mod ui;
mod repository;
#[macro_use]
mod log;
mod app_config;

lazy_static! {
    pub static ref SETTINGS: RwLock<AppConfig> = RwLock::new(Config::builder()
        // Add in `./config.{yml|toml}`
        .add_source(config::File::with_name("config"))
        // Add in settings from the environment (with a prefix of TRACK_WORK)
        // Eg.. `TRACK_WORK_DEBUG=1 ./target/app` would set the `debug` key
        .add_source(config::Environment::with_prefix("TRACK_WORK"))
        .build()
        .unwrap()
        .try_into()
        .expect("Config malformed"));
}

/// Demo
#[derive(Debug, FromArgs)]
struct Cli {
    /// time in ms between two ticks.
    #[argh(option, default = "250")]
    tick_rate: u64,
    /// whether unicode symbols are used to improve the overall look of the app
    #[argh(option, default = "true")]
    enhanced_graphics: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli: Cli = argh::from_env();
    let tick_rate = Duration::from_millis(cli.tick_rate);
    run(tick_rate, cli.enhanced_graphics)?;
    Ok(())
}