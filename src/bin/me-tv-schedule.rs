/*
 *  Me TV — It's TV for me computer.
 *
 *  A GTK+/GStreamer client for watching and recording DVB.
 *
 *  Copyright © 2018, 2019  Russel Winder
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

extern crate clap;
extern crate chrono;
extern crate exitcode;
#[cfg(test)]
extern crate rstest;
extern crate time;  // Need this for durations for chrono.

use std::process;

use clap::{Arg, App};

use chrono::{DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use time::{Duration, now};

fn parse_to_datetime(datum: &str) -> Result<NaiveDateTime, &str> {
    let datetime_patterns = [
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M",
        "%Y-%m-%d %H:%M",
    ];
    let time_patterns= [
        "%H:%M:%S",
        "%H:%M",
    ];
    for pattern in datetime_patterns.iter() {
        if let Ok(datetime) = NaiveDateTime::parse_from_str(datum, pattern) {
            return Ok(datetime.into());
        }
    }
    for pattern in time_patterns.iter() {
        if let Ok(time) = NaiveTime::parse_from_str(datum, pattern) {
            return Ok(Local::today().and_time(time).unwrap().naive_local())
        }
    };
    Err(datum)
}

fn main() {
    let matches = App::new("me-tv-schedule")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Russel Winder <russel@winder.org.uk>")
        .about("Schedule recording to create an MPEG4 file.

A channel name, a start time, a file path, and either an end time
or a duration must be provided.

Start and end times must be in ISO8601 format which means
either YYYY-MM-DD'T'hh:mm or YYYYMMDD'T'hhss.
")
        .arg(Arg::with_name("adapter")
            .short("a")
            .long("adapter")
            .value_name("NUMBER")
            .help("Sets the adapter number to use.")
            .takes_value(true)
            .default_value("0"))
        .arg(Arg::with_name("frontend")
            .short("f")
            .long("frontend")
            .value_name("NUMBER")
            .help("Sets the frontend number to use.")
            .takes_value(true)
            .default_value("0"))
        .arg(Arg::with_name("channel")
            .short("c")
            .long("channel")
            .value_name("CHANNEL")
            .help("Sets the channel name, must be specified, no default.")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("start_time")
            .short("s")
            .long("start-time")
            .value_name("DATE-TIME")
            .help("Sets the start date and time time of recording, ISO8601 format, must be specified, no default.")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("end_time")
            .short("e")
            .long("end-time")
            .value_name("DATE-TIME")
            .help("Sets the end date and time of recording, ISO8601 format, no default. This must be set if duration is not, but do not set both.")
            .takes_value(true)
            .conflicts_with("duration"))
        .arg(Arg::with_name("duration")
            .short("d")
            .long("duration")
            .value_name("TIME")
            .help("Sets the duration of recording in minutes, no default. This must be set unless end-time is, but do not set both.")
            .takes_value(true)
            .required_unless("end-time"))
        .arg(Arg::with_name("output")
            .short("o")
            .long("output")
            .value_name("PATH")
            .help("Path to output file, must be specified, no default.")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .help("sets verbose mode"))
        .get_matches();
    let be_verbose = matches.is_present("verbose");
    let adapter = matches.value_of("adapter").unwrap().parse::<u8>().expect("Couldn't parse adapter value as a positive integer.");
    let frontend = matches.value_of("frontend").unwrap().parse::<u8>().expect("Couldn't parse frontend value as a positive integer.");
    let channel = matches.value_of("channel").unwrap();
    let start_time = parse_to_datetime(matches.value_of("start_time").unwrap()).expect("Could not parse start time.");
    let end_time = match matches.value_of("end_time") {
        Some(e_t) => Some(parse_to_datetime(e_t).expect("Could not parse end time.")),
        None => None,
    };
    let duration = match matches.value_of("duration") {
        Some(d) => Some(Duration::minutes(d.parse::<i64>().expect("Couldn't parse the provided duration as an integer."))),
        None => None,
    };
    if end_time.is_none() && duration.is_none() {
        println!("You must specify either the end time or the duration of the recording.");
        process::exit(exitcode::USAGE);
    }
    if end_time.is_some() && duration.is_some() {
        if end_time.unwrap() - start_time != duration.unwrap() {
            println!("Both end time and duration were supplied but there was a conflict between them, just give one or the other.");
            process::exit(exitcode::USAGE);
        }
    }
    let duration = if end_time.is_some() {
        end_time.unwrap() - start_time
    } else {
        duration.unwrap()
    };
    let output_file = matches.value_of("output").unwrap();
    if be_verbose {
        println!(
            "Scheduling recording of channel '{}' at {:?} for {} minutes to file {} using adapter {}, frontend {}.",
            channel,
            start_time,
            duration.num_minutes(),
            output_file,
            adapter,
            frontend,
        );
    }
    let echo_process = process::Command::new("echo")
        .arg(format!(
            "me-tv-record --channel={} --duration={} --output={} --adapter={} --frontend={}",
            channel,
            duration.num_minutes(),
            output_file,
            adapter,
            frontend,
        ))
        .stdout(process::Stdio::piped())
        .spawn()
        .expect("Failed to start echo process.");
    let echo_pipe = echo_process.stdout.expect("Failed to open the echo process stdout.");
    let at_process = process::Command::new("at")
        .arg(format!("{}", start_time.time().format("%H:%M")))
        .arg(format!("{}", start_time.date().format("%Y-%m-%d")))
        .stdin(process::Stdio::from(echo_pipe))
        .spawn()
        .expect("Failed to start at process.");
}

#[cfg(test)]
mod test {
    use super::*;
    use rstest::rstest_parametrize;

    #[rstest_parametrize(
        datum, expected,
        case("2018-12-29T18:15:33", Unwrap("NaiveDate::from_ymd(2018, 12, 29).and_hms(18, 15, 33)")),
        case("2018-12-29 18:15:33", Unwrap("NaiveDate::from_ymd(2018, 12, 29).and_hms(18, 15, 33)")),
        case("2018-12-29T18:15", Unwrap("NaiveDate::from_ymd(2018, 12, 29).and_hms(18, 15, 00)")),
        case("2018-12-29 18:15", Unwrap("NaiveDate::from_ymd(2018, 12, 29).and_hms(18, 15, 00)")),
        case("18:15:13", Unwrap("Local::today().naive_local().and_hms(18, 15, 13)")),
        case("18:15", Unwrap("Local::today().naive_local().and_hms(18, 15, 00)")),
    )]
    fn parse_datetime_string(datum: &str, expected: NaiveDateTime) {
        match parse_to_datetime(datum) {
            Ok(result) => assert_eq!(result, expected),
            Err(e) => assert!(false,"failed to parse: {}", e),
        };
    }
}
