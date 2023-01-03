use chrono::{Datelike, Weekday};
use discord_log::Logger;
use kava_mysql::get_mysql_connection;
use mysql::prelude::Queryable;
use pubdata::*;
use schedule::*;
use std::fs;

fn main() {
	let logger = Logger::new();
	let contents = fs::read_to_string("../web/static/resources/json/PublicData.json")
		.expect("PublicData.json not found");
	let /*to use*/_pubdata: PublicData = serde_json::from_str(&contents).unwrap();
	let col_id = match chrono::offset::Local::now().date_naive().weekday() {
		Weekday::Sun => 1,
		Weekday::Mon => 2,
		Weekday::Tue => 3,
		Weekday::Wed => 4,
		Weekday::Thu => 5,
		Weekday::Fri => 6,
		Weekday::Sat => 7,
	};
	let row = match get_mysql_connection().query_map(
		"SELECT location, sun1, mon1, tue1, wed1, thu1, fri1, sat1 from schedule ORDER BY `id`",
		|(location, sun1, mon1, tue1, wed1, thu1, fri1, sat1)| -> [String; 8] {
			[location, sun1, mon1, tue1, wed1, thu1, fri1, sat1]
		},
	) {
		Ok(v) => {
			let mut day = ScheduleDay::empty();
			for row in v {
				let shifts: Vec<ScheduleShift> = serde_json::from_str(&row[col_id][..]).unwrap();
				let loc = ScheduleLocation {
					name: row[0].to_string(),
					shifts: shifts,
				};

				day.locations.push(loc);
			}
			day
		}
		Err(e) => {
			logger.panic(format!("daily/src/main.rs Select Error: {}", e.to_string()));
			panic!();
		}
	};

	// in progress. print for now
	for loc in row.locations {
		println!("LOCATION: {}", loc.name);
		for shift in loc.shifts {
			println!("{}: {}", shift.name, shift.bartender);
		}
	}
}
