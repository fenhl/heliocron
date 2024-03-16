use std::io::Write;
use std::result;

use chrono::Local;
use crossterm::{cursor, terminal, ExecutableCommand, QueueableCommand};

use super::{calc, errors, report};

type Result<T> = result::Result<T, errors::HeliocronError>;

pub fn display_report(solar_calculations: calc::SolarCalculations, json: bool) -> Result<()> {
    let report = report::Report::new(solar_calculations);
    let output = if json {
        serde_json::to_string(&report).unwrap()
    } else {
        report.to_string()
    };
    println!("{}", output);
    Ok(())
}

pub fn poll(solar_calculations: calc::SolarCalculations, watch: bool, json: bool) -> Result<()> {
    let mut report = report::PollReport::new(&solar_calculations);
    let output = if json {
        serde_json::to_string(&report).unwrap()
    } else {
        report.to_string()
    };

    if !watch {
        println!("{output}");
    } else {
        if !json {
            println!("Displaying solar calculations in real time. Press ctrl+C to cancel.\n");
        }

        // Set up stdout and make a record of the current cursor location. We unwrap
        let mut stdout = std::io::stdout();
        stdout.queue(cursor::SavePosition).unwrap();
        stdout.execute(cursor::Hide).unwrap();

        loop {
            if json {
                println!("{}", serde_json::to_string(&report).unwrap());
            } else {
                stdout.queue(cursor::RestorePosition).unwrap();
                stdout
                    .queue(terminal::Clear(terminal::ClearType::FromCursorDown))
                    .unwrap();
                stdout.write_all(report.to_string().as_bytes()).unwrap();
                stdout.flush().unwrap();
            }

            std::thread::sleep(std::time::Duration::from_secs(1));

            let now = Local::now();
            let now = now.with_timezone(now.offset());

            let calcs = solar_calculations.refresh(now);

            report = report::PollReport::new(&calcs);
        }
    }

    Ok(())
}
