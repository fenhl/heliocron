use chrono::prelude::*;

fn main() -> Result<(), String> {
    println!("{}", heliocron::domain::DayPart::from_elevation_angle(
        heliocron::calc::SolarCalculations::new(
            Utc::now().into(),
            heliocron::domain::Coordinates {
                latitude: heliocron::domain::Latitude::new(49.8077)?,
                longitude: heliocron::domain::Longitude::new(7.9647)?,
            },
        ).solar_elevation(),
    ));
    Ok(())
}
