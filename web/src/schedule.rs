use mysql::prelude::*;
use mysql::*;
use rocket::http::Status;
use rocket::response::content::RawJson;
use rocket::response::status;
use rocket::serde::{json::Json, Deserialize, Serialize};
use std::result::Result;

#[derive(Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct Schedule {
	week1: ScheduleWeek,
	week2: ScheduleWeek,
}

impl Schedule {
	fn empty() -> Schedule {
		Schedule {
			week1: ScheduleWeek::empty(),
			week2: ScheduleWeek::empty(),
		}
	}

	fn get_week(&mut self, number: i8) -> &mut ScheduleWeek {
		match number {
			1 => &mut self.week1,
			2 => &mut self.week2,
			_ => panic!("Invalid week: {}", number),
		}
	}
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct ScheduleWeek {
	sun: ScheduleDay,
	mon: ScheduleDay,
	tue: ScheduleDay,
	wed: ScheduleDay,
	thu: ScheduleDay,
	fri: ScheduleDay,
	sat: ScheduleDay,
}

impl ScheduleWeek {
	fn empty() -> ScheduleWeek {
		ScheduleWeek {
			sun: ScheduleDay::empty(),
			mon: ScheduleDay::empty(),
			tue: ScheduleDay::empty(),
			wed: ScheduleDay::empty(),
			thu: ScheduleDay::empty(),
			fri: ScheduleDay::empty(),
			sat: ScheduleDay::empty(),
		}
	}

	fn days() -> Vec<String> {
		vec![
			"sun".to_string(),
			"mon".to_string(),
			"tue".to_string(),
			"wed".to_string(),
			"thu".to_string(),
			"fri".to_string(),
			"sat".to_string(),
		]
	}

	fn get_day(&mut self, day: &String) -> &mut ScheduleDay {
		match &day[..] {
			"sun" => &mut self.sun,
			"mon" => &mut self.mon,
			"tue" => &mut self.tue,
			"wed" => &mut self.wed,
			"thu" => &mut self.thu,
			"fri" => &mut self.fri,
			"sat" => &mut self.sat,
			_ => panic!("Invalid day: {}", day),
		}
	}
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct ScheduleDay {
	locations: Vec<ScheduleLocation>,
}

impl ScheduleDay {
	fn empty() -> ScheduleDay {
		ScheduleDay { locations: vec![] }
	}

	fn get_location_index_by_name(&mut self, name: String) -> usize {
		let mut i = 0;
		for location in &mut self.locations {
			if location.name == name {
				return i;
			}
			i += 1;
		}
		self.locations.push(ScheduleLocation {
			name,
			shifts: vec![],
		});
		self.locations.len() - 1
	}
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct ScheduleLocation {
	name: String,
	shifts: Vec<ScheduleShift>,
}

impl ScheduleLocation {
	fn get_shift_index_by_name(&mut self, name: String) -> usize {
		let mut i = 0;
		for shift in &mut self.shifts {
			if shift.name == name {
				return i;
			}
			i += 1;
		}
		self.shifts.push(ScheduleShift {
			name,
			bartender: "".to_string(),
		});
		self.shifts.len() - 1
	}
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct ScheduleShift {
	name: String,
	bartender: String,
}

#[derive(Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct SchedulePostRequest {
	verify: super::Credentials,
	schedule: Schedule,
}

#[derive(Clone, Serialize)]
#[serde(crate = "rocket::serde")]
struct ScheduleRow {
	location: String,
	sun1: String,
	mon1: String,
	tue1: String,
	wed1: String,
	thu1: String,
	fri1: String,
	sat1: String,
	sun2: String,
	mon2: String,
	tue2: String,
	wed2: String,
	thu2: String,
	fri2: String,
	sat2: String,
}

impl ScheduleRow {
	fn empty(location: &String) -> ScheduleRow {
		ScheduleRow {
			location: location.to_string(),
			sun1: String::new(),
			mon1: String::new(),
			tue1: String::new(),
			wed1: String::new(),
			thu1: String::new(),
			fri1: String::new(),
			sat1: String::new(),
			sun2: String::new(),
			mon2: String::new(),
			tue2: String::new(),
			wed2: String::new(),
			thu2: String::new(),
			fri2: String::new(),
			sat2: String::new(),
		}
	}

	fn set_day(&mut self, day: &str, val: String) {
		match day {
			"sun1" => self.sun1 = val,
			"mon1" => self.mon1 = val,
			"tue1" => self.tue1 = val,
			"wed1" => self.wed1 = val,
			"thu1" => self.thu1 = val,
			"fri1" => self.fri1 = val,
			"sat1" => self.sat1 = val,
			"sun2" => self.sun2 = val,
			"mon2" => self.mon2 = val,
			"tue2" => self.tue2 = val,
			"wed2" => self.wed2 = val,
			"thu2" => self.thu2 = val,
			"fri2" => self.fri2 = val,
			"sat2" => self.sat2 = val,
			_ => panic!("Invalid day: {}", day),
		};
	}

	fn get_day(&self, day: &str) -> &String {
		match day {
			"sun1" => &self.sun1,
			"mon1" => &self.mon1,
			"tue1" => &self.tue1,
			"wed1" => &self.wed1,
			"thu1" => &self.thu1,
			"fri1" => &self.fri1,
			"sat1" => &self.sat1,
			"sun2" => &self.sun2,
			"mon2" => &self.mon2,
			"tue2" => &self.tue2,
			"wed2" => &self.wed2,
			"thu2" => &self.thu2,
			"fri2" => &self.fri2,
			"sat2" => &self.sat2,
			_ => panic!("Invalid day: {}", day),
		}
	}
}

#[rocket::put(
	"/api/schedule_update",
	format = "application/json",
	data = "<request>"
)]
pub fn schedule_update(request: Json<SchedulePostRequest>) -> status::Custom<RawJson<String>> {
	let result = |request: Json<SchedulePostRequest>| -> Result<(), (Status, String)> {
		request.verify.authenticate()?;
		let mut conn = super::get_mysql_connection()?;
		if conn.query_drop("TRUNCATE schedule").is_err() {
			return Err((Status::InternalServerError, "Truncate error".to_string()));
		}
		let mut locations = vec![];
		for loc in &request.schedule.week1.sun.locations {
			locations.push(loc.name.to_string());
		}

		let mut new_rows = vec![];

		for loc in &locations {
			let mut new_row = ScheduleRow::empty(loc);
			for week_i in vec![1, 2] {
				for day_i in ScheduleWeek::days() {
					for day_loc in &request
						.clone()
						.schedule
						.get_week(week_i)
						.get_day(&day_i)
						.locations
					{
						if &day_loc.name == loc {
							let mut day_string = day_i.to_string();
							day_string.push_str(&week_i.to_string());
							new_row.set_day(
								&day_string,
								serde_json::to_string(&day_loc.shifts).unwrap(),
							);
							break;
						}
					}
				}
			}

			new_rows.push(new_row);
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
		match sql_result {
			Ok(_) => Ok(()),
			Err(v) => Err((Status::InternalServerError, v.to_string())),
		}
	};
	match result(request) {
		Ok(_) => status::Custom(Status::Ok, RawJson("{\"success\":true}".to_owned())),
		Err(e) => status::Custom(e.0, RawJson(e.1)),
	}
}

#[rocket::post(
	"/api/schedule_get",
	format = "application/json",
	data = "<input_creds>"
)]
pub fn schedule_get(input_creds: Json<super::Credentials>) -> status::Custom<RawJson<String>> {
	let result = |request: Json<super::Credentials>| -> Result<Schedule, (Status, String)> {
		request.authenticate()?;
		let mut conn = super::get_mysql_connection()?;
		let week1_result = conn.query_map(
			"SELECT location, sun1, mon1, tue1, wed1, thu1, fri1, sat1 from schedule ORDER BY `id`",
			|(location, sun1, mon1, tue1, wed1, thu1, fri1, sat1)| {
				(location, sun1, mon1, tue1, wed1, thu1, fri1, sat1)
			},
		);

		let week2_result: Result<
			Vec<(String, String, String, String, String, String, String)>,
			Error,
		> = conn.query_map(
			"SELECT sun2, mon2, tue2, wed2, thu2, fri2, sat2 from schedule ORDER BY `id`",
			|(sun2, mon2, tue2, wed2, thu2, fri2, sat2)| (sun2, mon2, tue2, wed2, thu2, fri2, sat2),
		);
		if week1_result.is_err() || week2_result.is_err() {
			return Err((Status::from_code(500).unwrap(), "MySQL Error".to_string()));
		}

		let w1 = week1_result.ok().unwrap();
		let w2 = week2_result.ok().unwrap();

		let mut rows = vec![];
		let mut i = 0;
		for row in w1 {
			rows.push(ScheduleRow {
				location: row.0,
				sun1: row.1,
				mon1: row.2,
				tue1: row.3,
				wed1: row.4,
				thu1: row.5,
				fri1: row.6,
				sat1: row.7,
				sun2: w2[i].0.to_string(),
				mon2: w2[i].1.to_string(),
				tue2: w2[i].2.to_string(),
				wed2: w2[i].3.to_string(),
				thu2: w2[i].4.to_string(),
				fri2: w2[i].5.to_string(),
				sat2: w2[i].6.to_string(),
			});
			i += 1;
		}
		let mut schedule = Schedule::empty();
		for row in &rows {
			for day in ScheduleWeek::days() {
				schedule
					.week1
					.get_day(&day)
					.locations
					.push(ScheduleLocation {
						name: row.location.to_string(),
						shifts: vec![],
					})
			}
		}

		for week_i in [1, 2] {
			for day_i in ScheduleWeek::days() {
				let mut day_string = day_i.to_string();
				day_string.push_str(&week_i.to_string());
				for row in &rows {
					let shifts: Vec<ScheduleShift> =
						serde_json::from_str(row.get_day(&day_string[..])).unwrap();
					let day = schedule.get_week(week_i).get_day(&day_i);
					let loc_i = day.get_location_index_by_name(row.location.to_string());
					for shift in shifts {
						let shift_i = day.locations[loc_i].get_shift_index_by_name(shift.name);
						day.locations[loc_i].shifts[shift_i].bartender = shift.bartender;
					}
				}
			}
		}

		Ok(schedule)
	};

	match result(input_creds) {
		Ok(v) => status::Custom(Status::Ok, RawJson(serde_json::to_string(&v).unwrap())),
		Err(e) => status::Custom(e.0, RawJson(e.1)),
	}
}
