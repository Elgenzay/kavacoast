use chrono::{Datelike, Weekday};
use discord_log::Logger;
use kava_mysql::get_mysql_connection;
use mysql::prelude::Queryable;
use pubdata::*;
use schedule::*;
use std::fs;

struct DayMeta {
	column_index: usize,
	friendly_name: String,
}

impl DayMeta {
	fn new(column_index: usize, friendly_name: &str) -> DayMeta {
		DayMeta {
			column_index,
			friendly_name: friendly_name.to_string(),
		}
	}
}

fn main() {
	let logger = Logger::new();
	let contents = fs::read_to_string("../web/static/resources/json/PublicData.json")
		.expect("PublicData.json not found");
	let pubdata: PublicData = serde_json::from_str(&contents).unwrap();
	let daymeta = match chrono::offset::Local::now().date_naive().weekday() {
		Weekday::Sun => DayMeta::new(1, "Sunday"),
		Weekday::Mon => DayMeta::new(2, "Monday"),
		Weekday::Tue => DayMeta::new(3, "Tuesday"),
		Weekday::Wed => DayMeta::new(4, "Wednesday"),
		Weekday::Thu => DayMeta::new(5, "Thursday"),
		Weekday::Fri => DayMeta::new(6, "Friday"),
		Weekday::Sat => DayMeta::new(7, "Saturday"),
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
				let shifts: Vec<ScheduleShift> =
					serde_json::from_str(&row[daymeta.column_index][..]).unwrap();
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
	let mut any = false;
	let mut reactions = vec![];
	let mut message = String::new();
	message.push_str(&daymeta.friendly_name);
	message.push_str("\n-  -  -  -  -  -  -  -  -  -");
	for loc in row.locations {
		let mut has_bartender = false;
		for shift in &loc.shifts {
			if !shift.bartender.is_empty() {
				has_bartender = true;
			}
		}
		if has_bartender {
			any = true;
			let pubdataloc = pubdata.get_location_by_name(&loc.name).unwrap();
			reactions.push(&pubdataloc.emoji);
			message.push_str("\n\n");
			message.push_str(&pubdataloc.friendly_name[..]);
			for shift in loc.shifts {
				if shift.bartender.is_empty() {
					continue;
				}
				message.push_str("\n");
				let pubdatashift = pubdata.get_shift_by_name(&shift.name).unwrap();
				message.push_str(
					&format!(
						"{}: {} ({})",
						&pubdatashift.friendly_name[..],
						&shift.bartender,
						&pubdatashift.description
					)[..],
				);
			}
		}
	}
	if any {
		logger.log_schedule(message, reactions);
	}
}
