use std::{fs, path::PathBuf, result};

use chrono::{DateTime, Datelike, FixedOffset, Local, NaiveDate, TimeZone};
use clap::{Parser, Subcommand};
use serde::Deserialize;

use super::{domain, errors::HeliocronError};

type Result<T, E = HeliocronError> = result::Result<T, E>;

#[derive(Parser)]
#[clap(version, about)]
struct Cli {
    /// Set the date for which the calculations should be run. If specified, it should be in 'yyyy-mm-dd' format, otherwise it defaults
    /// to the the current local date
    #[clap(
        short = 'd',
        long = "date",
        value_parser=parse_date,
        default_value_t=Local::today().naive_local()
    )]
    date: NaiveDate,

    /// Set the time zone. If specified, it should be in the format '[+/-]HH:MM', otherwise it defaults to the current local time zone
    #[clap(short = 't', long = "time-zone", allow_hyphen_values = true, value_parser=parse_tz, default_value_t=*Local::today().offset())]
    time_zone: FixedOffset,

    /// Set the latitude in decimal degrees. Positive values to the north; negative values to the south. Defaults to '51.4769' if not
    /// otherwise specified here or in ~/.config/heliocron.toml.
    #[clap(short = 'l', long = "latitude", requires = "longitude", allow_hyphen_values = true, value_parser = domain::Latitude::parse)]
    latitude: Option<domain::Latitude>,

    /// Set the longitude in decimal degrees. Positive values to the east; negative values to the west. Defaults to '-0.0005' if not
    /// otherwise specified here or in ~/.config/heliocron.toml
    #[clap(short = 'o', long = "longitude", requires = "latitude", allow_hyphen_values = true, value_parser = domain::Longitude::parse)]
    longitude: Option<domain::Longitude>,

    #[clap(subcommand)]
    subcommand: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Produce a full set of sunrise, sunset and other related times for the given date and location
    Report {
        /// Set the output format to machine-readable JSON. If this flag is not present, the report will be displayed in the terminal as a block of human-readable text
        #[clap(long = "json")]
        json: bool,
    },

    /// Display real time data pertaining to the Sun at the current local time
    Poll {
        /// Run the program constantly, updating the values every second
        #[clap(long = "watch")]
        watch: bool,

        /// Set the output format to machine-readable JSON. If this flag is not present, the report will be displayed in the terminal as a block of human-readable text
        #[clap(long = "json")]
        json: bool,
    },
}

fn parse_date(date: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|_| format!("Invalid date - must be in the format 'yyyy-mm-dd'. Found '{date}'"))
}

fn parse_tz(tz: &str) -> Result<chrono::FixedOffset, String> {
    // Use chrono's own parsing function to validate the provided time zone.
    let date = chrono::DateTime::parse_from_str(&format!("2022-01-01T00:00:00{}", tz), "%FT%T%:z")
        .map_err(|_| {
            format!(
                "Invalid time zone - expected the format '[+|-]HH:MM' between '-23:59' and '+23:59'. Found '{tz}'"
            )
        })?;
    Ok(*date.offset())
}

#[derive(Debug, Deserialize)]
struct RawFileConfig {
    latitude: Option<f64>,
    longitude: Option<f64>,
}

/// Container for all necessary runtime configuration.
pub struct Config {
    pub coordinates: domain::Coordinates,
    pub date: DateTime<FixedOffset>,
    pub action: domain::Action,
}

/// Parse all configuration streams into one valid runtime configuration. Where supported, arguments passed over the
/// command line take precedence over values found in configuration files, which, in turn, takes precedence over
/// any hard coded default values.
pub fn parse_config() -> Result<Config, HeliocronError> {
    let cli_args = Cli::parse();

    let coordinates = {
        // First try the command line arguments...
        if let (Some(lat), Some(lon)) = (cli_args.latitude, cli_args.longitude) {
            domain::Coordinates::new(lat, lon)
        } else {
            // ...failing that, check if the coordinates are set in a config file...
            dirs::config_dir()
                .map(|path| path.join("heliocron.toml"))
                .filter(|path| path.exists())
                .map(|path| parse_local_config(&path))
                .and_then(|res| {
                    match res {
                        Ok(coords) => Some(coords),
                        Err(e) => {
                            eprintln!("Warning - couldn't parse configuration file due to the following reason: {}\n. Proceeding with default coordinates.", e);
                            None
                        }
                        }
                })
                .unwrap_or_else(|| {
                    // ...otherwise default to some hardcoded values. Safe to unwrap because we know these values are valid.
                    domain::Coordinates::new(
                        domain::Latitude::new(51.4769).unwrap(),
                        domain::Longitude::new(-0.0005).unwrap(),
                    )
                })
        }
    };

    let date = match cli_args.subcommand {
        Command::Poll { .. } => {
            let now = Local::now();
            now.with_timezone(now.offset())
        }
        _ => cli_args
            .time_zone
            .ymd(
                cli_args.date.year(),
                cli_args.date.month(),
                cli_args.date.day(),
            )
            .and_hms(12, 0, 0),
    };

    let action = match cli_args.subcommand {
        Command::Report { json } => domain::Action::Report { json },
        Command::Poll { watch, json } => domain::Action::Poll { watch, json },
    };

    Ok(Config {
        coordinates,
        date,
        action,
    })
}

fn parse_local_config(path: &PathBuf) -> Result<domain::Coordinates, String> {
    let config_file = fs::read(path).map_err(|_| "Failed to read config file path".to_string())?;
    let toml_config = toml::from_slice::<RawFileConfig>(&config_file).map_err(|e| e.to_string())?;

    let (lat, lon) = match (toml_config.latitude, toml_config.longitude) {
        (Some(lat), Some(lon)) => Ok((lat, lon)),
        (Some(_lat), None) => Err("Missing longitude".to_string()),
        (None, Some(_lon)) => Err("Missing latitude".to_string()),
        (None, None) => Err("Missing latitude and longitude".to_string()),
    }?;

    let lat = domain::Latitude::new(lat)?;
    let lon = domain::Longitude::new(lon)?;

    Ok(domain::Coordinates::new(lat, lon))
}
