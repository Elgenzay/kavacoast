use chrono::{Datelike, Weekday};
use discord_log::Logger;
use kava_mysql::get_mysql_connection;
use mysql::{params, prelude::Queryable};
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

pub fn daily() {
	let logger = Logger::new();
	let contents = fs::read_to_string("../web/static/resources/json/PublicData.json")
		.expect("PublicData.json not found");
	let pubdata: PublicData = match serde_json::from_str(&contents) {
		Ok(v) => v,
		Err(_) => logger.panic("Error parsing PublicData.json".to_owned()),
	};
	let daymeta = match chrono::offset::Utc::now().date_naive().weekday() {
		Weekday::Sun => DayMeta::new(1, "Sunday"),
		Weekday::Mon => DayMeta::new(2, "Monday"),
		Weekday::Tue => DayMeta::new(3, "Tuesday"),
		Weekday::Wed => DayMeta::new(4, "Wednesday"),
		Weekday::Thu => DayMeta::new(5, "Thursday"),
		Weekday::Fri => DayMeta::new(6, "Friday"),
		Weekday::Sat => DayMeta::new(7, "Saturday"),
	};
	let mut conn = match get_mysql_connection() {
		Ok(v) => v,
		Err(e) => logger.panic(format!("daily() MySQL connect error: {}", e)),
	};
	let row = match conn.query_map(
		"SELECT location, sun1, mon1, tue1, wed1, thu1, fri1, sat1 from schedule ORDER BY `id`",
		|(location, sun1, mon1, tue1, wed1, thu1, fri1, sat1)| -> [String; 8] {
			[location, sun1, mon1, tue1, wed1, thu1, fri1, sat1]
		},
	) {
		Ok(v) => {
			let mut day = ScheduleDay::empty();
			for row in v {
				let shifts: Vec<ScheduleShift> =
					match serde_json::from_str(&row[daymeta.column_index][..]) {
						Ok(v) => v,
						Err(_) => logger.panic(format!("daily(): Error parsing day")),
					};
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
		}
	};

	let bartenders: Vec<(String, u64)> = match conn.query("SELECT name, discord_id FROM bartenders")
	{
		Ok(v) => v,
		Err(e) => logger.panic(format!("daily() MySQL select error: {}", e.to_string())),
	};

	let mut any = false;
	let mut reactions = vec![];
	let mut message = String::new();
	message.push_str("@silent ");
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
			let pubdataloc = match pubdata.get_location_by_name(&loc.name) {
				Some(v) => v,
				None => logger.panic(
					"schedule_notify/src/lib.rs: get_location_by_name returned None".to_owned(),
				),
			};
			reactions.push(&pubdataloc.emoji);
			message.push_str("\n\n");
			message.push_str(&pubdataloc.friendly_name[..]);
			for shift in loc.shifts {
				if shift.bartender.is_empty() {
					continue;
				}
				message.push_str("\n");
				let pubdatashift = match pubdata.get_shift_by_name(&shift.name) {
					Some(v) => v,
					None => logger.panic(
						"schedule_notify/src/lib.rs: get_shift_by_name returned None".to_owned(),
					),
				};
				let mut bt_id = shift.bartender.to_string();
				for bartender in &bartenders {
					if &bartender.0 == &shift.bartender {
						if bartender.1 != 0 {
							bt_id = format!("<@{}>", bartender.1.to_string());
						}
						break;
					}
				}
				message.push_str(
					&format!(
						"{}:  {}  ({})",
						&pubdatashift.friendly_name[..],
						bt_id,
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

pub fn weekly() {
	let logger = Logger::new();
	let contents = fs::read_to_string("../web/static/resources/json/PublicData.json")
		.expect("PublicData.json not found");
	let pubdata: PublicData = match serde_json::from_str(&contents) {
		Ok(v) => v,
		Err(_) => logger.panic("weekly(): Error parsing PublicData.json".to_owned()),
	};
	let mut conn = match get_mysql_connection() {
		Ok(v) => v,
		Err(e) => logger.panic(format!("weekly() MySQL connect error: {}", e)),
	};
	let w2: Vec<(
		String,
		String,
		String,
		String,
		String,
		String,
		String,
		String,
	)> = conn
		.query_map(
			"SELECT location, sun2, mon2, tue2, wed2, thu2, fri2, sat2 from schedule ORDER BY `id`",
			|(location, sun2, mon2, tue2, wed2, thu2, fri2, sat2)| {
				(location, sun2, mon2, tue2, wed2, thu2, fri2, sat2)
			},
		)
		.expect("Select error");
	if conn.query_drop("TRUNCATE schedule").is_err() {
		logger.panic("weekly/src/main.rs: Truncate error".to_owned());
	}
	let mut new_rows = vec![];
	let mut empty_day_cell_obj = vec![];

	for shift in pubdata.shifts {
		empty_day_cell_obj.push(ScheduleShift {
			name: shift.name.to_string(),
			bartender: String::new(),
		});
	}
	let newday = serde_json::to_string(&empty_day_cell_obj).unwrap();
	for location in pubdata.locations {
		for row in &w2 {
			if row.0 == location.name {
				new_rows.push(ScheduleRow {
					location: location.name.to_string(),
					sun1: row.1.to_string(),
					mon1: row.2.to_string(),
					tue1: row.3.to_string(),
					wed1: row.4.to_string(),
					thu1: row.5.to_string(),
					fri1: row.6.to_string(),
					sat1: row.7.to_string(),
					sun2: newday.to_string(),
					mon2: newday.to_string(),
					tue2: newday.to_string(),
					wed2: newday.to_string(),
					thu2: newday.to_string(),
					fri2: newday.to_string(),
					sat2: newday.to_string(),
				});
			}
		}
	}

	let sql_result = conn.exec_batch("INSERT INTO schedule (
		location, sun1, mon1, tue1, wed1, thu1, fri1, sat1, sun2, mon2, tue2, wed2, thu2, fri2, sat2
	) VALUES (
		:location, :sun1, :mon1, :tue1, :wed1, :thu1, :fri1, :sat1, :sun2, :mon2, :tue2, :wed2, :thu2, :fri2, :sat2
	)", new_rows.iter().map(|r| params! {
			"location" => &r.location,
			"sun1" => &r.sun1,
			"mon1" => &r.mon1,
			"tue1" => &r.tue1,
			"wed1" => &r.wed1,
			"thu1" => &r.thu1,
			"fri1" => &r.fri1,
			"sat1" => &r.sat1,
			"sun2" => &r.sun2,
			"mon2" => &r.mon2,
			"tue2" => &r.tue2,
			"wed2" => &r.wed2,
			"thu2" => &r.thu2,
			"fri2" => &r.fri2,
			"sat2" => &r.sat2,
		})
	);
	if sql_result.is_err() {
		logger.panic("weekly/src/main.rs: Insert error".to_owned());
	}
	logger.log_message("Week cycled successfully.".to_owned());
}
