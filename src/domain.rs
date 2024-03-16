use std::fmt;
use std::ops::RangeInclusive;

use chrono::{DateTime, FixedOffset, NaiveTime};

/// An enumeration of the different parts of the day. Not all of them necessarily occur during a
/// given 24-hour period.
pub enum DayPart {
    Day,
    CivilTwilight,
    NauticalTwilight,
    AstronomicalTwilight,
    Night,
}

impl DayPart {
    pub fn from_elevation_angle(angle: f64) -> Self {
        if angle < -18.0 {
            Self::Night
        } else if angle < -12.0 {
            Self::AstronomicalTwilight
        } else if angle < -6.0 {
            Self::NauticalTwilight
        } else if angle < 0.833 {
            Self::CivilTwilight
        } else {
            Self::Day
        }
    }
}

impl fmt::Display for DayPart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Day => "Day",
                Self::CivilTwilight => "Civil Twilight",
                Self::NauticalTwilight => "Nautical Twilight",
                Self::AstronomicalTwilight => "Astronomical Twilight",
                Self::Night => "Night",
            }
        )
    }
}

/// An enumeration of parsed commands.
pub enum Action {
    Report {
        json: bool,
    },
    Poll {
        watch: bool,
        json: bool,
    },
}

/// A newtype representing an optional datetime.
///
/// This allows us to provide custom serialization methods when converting to a String or JSON.
#[derive(Debug)]
pub struct EventTime(pub Option<DateTime<FixedOffset>>);

impl EventTime {
    pub fn new(datetime: Option<DateTime<FixedOffset>>) -> Self {
        Self(datetime)
    }

    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }

    pub fn time(&self) -> Option<NaiveTime> {
        self.0.map(|dt| dt.time())
    }
}

impl fmt::Display for EventTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self.0 {
                Some(datetime) => datetime.to_string(),
                None => "Never".to_string(),
            }
        )
    }
}

/// Newtype wrapper for validating an altitude between -90.0 and 90.0.
#[derive(Clone)]
pub struct Altitude(f64);

impl Altitude {
    pub fn new(alt: f64) -> Result<Self, String> {
        if (-90.0..=90.0).contains(&alt) {
            Ok(Self(alt))
        } else {
            Err(format!(
                "Expected a number between -90.0 and 90.0. Found '{alt}'"
            ))
        }
    }

    pub fn parse(alt: &str) -> Result<Self, String> {
        match alt.parse() {
            Ok(alt) => Self::new(alt),
            Err(alt) => Err(format!(
                "Expected a number between -90.0 and 90.0. Found '{alt}'"
            )),
        }
    }
}

impl From<f64> for Altitude {
    fn from(alt: f64) -> Self {
        Self::new(alt).unwrap()
    }
}

impl std::ops::Deref for Altitude {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A list of plain event names supported by the command line interface.
#[derive(Clone)]
pub enum RawEventName {
    Sunrise,
    Sunset,
    CivilDawn,
    CivilDusk,
    NauticalDawn,
    NauticalDusk,
    AstronomicalDawn,
    AstronomicalDusk,
    CustomAM,
    CustomPM,
    SolarNoon,
}

/// An enumeration of possible event names, with required data attached.
///
/// For example, CustomAM/PM here include the custom altitude, in contrast to
/// `RawEventName` where that data is absent.
pub enum EventName {
    Sunrise,
    Sunset,
    CivilDawn,
    CivilDusk,
    NauticalDawn,
    NauticalDusk,
    AstronomicalDawn,
    AstronomicalDusk,
    CustomAM(Altitude),
    CustomPM(Altitude),
    SolarNoon,
}

/// The set of possible directions of travel for a celestial object relative to the obeserver, i.e.
/// either ascending or descending.
pub enum Direction {
    Ascending,
    Descending,
}

/// Events which occur when the Sun reaches a specific elevation relative to the horizon.
///
/// For example, sunrise always occurs when the centre of the Sun is 0.833 degrees below the horizon.
pub struct FixedElevationEvent {
    pub degrees_below_horizon: Altitude,
    pub solar_direction: Direction,
}

impl FixedElevationEvent {
    pub fn new(degrees_below_horizon: Altitude, solar_direction: Direction) -> Self {
        Self {
            degrees_below_horizon,
            solar_direction,
        }
    }
}

/// Events which occur when the Sun is at a variable elevation.
///
/// For example, solar noon occurs at the maximum solar elevation, which varies based on time and location.
pub enum VariableElevationEvent {
    SolarNoon,
}

/// Any supported solar event.
///
/// Some events, such as sunrise and sunset, occur when the Sun is at a specific altitude relative to the horizon,
/// but other events, such as solar noon, occur not at a fixed altitude, but a variable one. Each of these has a
/// different way of calculating the time of the event, hence they are separated into two variants.
pub enum Event {
    Fixed(FixedElevationEvent),
    Variable(VariableElevationEvent),
}

impl Event {
    pub fn from_event_name(event: EventName) -> Self {
        // We can just use `.into()` (a method which can panic) for these float conversions because we can manually
        // verify that all of them are valid altitudes.
        match event {
            EventName::Sunrise => {
                Self::Fixed(FixedElevationEvent::new(0.833.into(), Direction::Ascending))
            }
            EventName::Sunset => Self::Fixed(FixedElevationEvent::new(
                0.833.into(),
                Direction::Descending,
            )),
            EventName::CivilDawn => {
                Self::Fixed(FixedElevationEvent::new(6.0.into(), Direction::Ascending))
            }
            EventName::CivilDusk => {
                Self::Fixed(FixedElevationEvent::new(6.0.into(), Direction::Descending))
            }
            EventName::NauticalDawn => {
                Self::Fixed(FixedElevationEvent::new(12.0.into(), Direction::Ascending))
            }
            EventName::NauticalDusk => {
                Self::Fixed(FixedElevationEvent::new(12.0.into(), Direction::Descending))
            }
            EventName::AstronomicalDawn => {
                Self::Fixed(FixedElevationEvent::new(18.0.into(), Direction::Ascending))
            }
            EventName::AstronomicalDusk => {
                Self::Fixed(FixedElevationEvent::new(18.0.into(), Direction::Descending))
            }
            EventName::CustomAM(alt) => {
                Self::Fixed(FixedElevationEvent::new(alt, Direction::Ascending))
            }
            EventName::CustomPM(alt) => {
                Self::Fixed(FixedElevationEvent::new(alt, Direction::Descending))
            }
            EventName::SolarNoon => Self::Variable(VariableElevationEvent::SolarNoon),
        }
    }
}

const LATITUDE_RANGE: RangeInclusive<f64> = RangeInclusive::new(-90.0, 90.0);
const LONGITUDE_RANGE: RangeInclusive<f64> = RangeInclusive::new(-180.0, 180.0);

/// Represents a latitude in decimal degrees. Valid values are from -90.0..=+90.0.
/// Positive values are to the north, whilst negative values are to the south.
#[derive(PartialEq, Debug, Clone)]
pub struct Latitude(f64);

impl Latitude {
    /// Create a new instance of `Latitude` from an f64.
    pub fn new(value: f64) -> Result<Self, String> {
        match LATITUDE_RANGE.contains(&value) {
            true => Ok(Self(value)),
            false => Err(format!(
                "Latitude must be between -90.0 and 90.0, inclusive. Found `{value}`."
            )),
        }
    }

    /// Create a new instance of `Latitude` from an &str, such as when parsing command line
    /// arguments.
    pub fn parse(value: &str) -> Result<Self, String> {
        value
            .parse()
            .map_err(|_| {
                format!("Latitude must be between -90.0 and 90.0, inclusive. Found `{value}`.")
            })
            .and_then(Self::new)
    }
}

impl fmt::Display for Latitude {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Deref for Latitude {
    type Target = f64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Represents a longitude in decimal degrees. Valid values are from -180.0..=+180.0.
/// Positive values are to the east, whilst negative values are to the west.
#[derive(PartialEq, Debug, Clone)]
pub struct Longitude(f64);

impl Longitude {
    /// Create a new `Longitude` from an f64.
    pub fn new(value: f64) -> Result<Self, String> {
        match LONGITUDE_RANGE.contains(&value) {
            true => Ok(Self(value)),
            false => Err(format!(
                "Longitude must be between -180.0 and 180.0, inclusive. Found '{value}'."
            )),
        }
    }

    /// Create a new instance of `Longitude` from an &str, such as when parsing command line
    /// arguments.
    pub fn parse(value: &str) -> Result<Self, String> {
        value
            .parse()
            .map_err(|_| {
                format!("Longitude must be between -180.0 and 180.0, inclusive. Found `{value}`.")
            })
            .and_then(Self::new)
    }
}

impl fmt::Display for Longitude {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Deref for Longitude {
    type Target = f64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Represents poisition on a map described by a latitude and longitude.
#[derive(Debug, PartialEq, Clone)]
pub struct Coordinates {
    pub latitude: Latitude,
    pub longitude: Longitude,
}

impl Coordinates {
    pub fn new(latitude: Latitude, longitude: Longitude) -> Self {
        Self {
            latitude,
            longitude,
        }
    }
}
