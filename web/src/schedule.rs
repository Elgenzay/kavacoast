use kava_mysql::get_mysql_connection;
use mysql::prelude::Queryable;
use mysql::{params, serde_json};
use rocket::http::Status;
use rocket::response::content::RawJson;
use rocket::response::status;
use rocket::serde::{json::Json, Deserialize};
use schedule::*;
use std::result::Result;

#[derive(Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct SchedulePostRequest {
	verify: super::Credentials,
	schedule: schedule::Schedule,
}

#[rocket::put(
	"/api/schedule_update",
	format = "application/json",
	data = "<request>"
)]
pub fn schedule_update(request: Json<SchedulePostRequest>) -> status::Custom<RawJson<String>> {
	let result = |request: Json<SchedulePostRequest>| -> Result<(), (Status, String)> {
		request.verify.authenticate()?;
		let mut conn = get_mysql_connection();
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
		let mut conn = get_mysql_connection();
		let week1_result = conn.query_map(
			"SELECT location, sun1, mon1, tue1, wed1, thu1, fri1, sat1 from schedule ORDER BY `id`",
			|(location, sun1, mon1, tue1, wed1, thu1, fri1, sat1)| {
				(location, sun1, mon1, tue1, wed1, thu1, fri1, sat1)
			},
		);

		let week2_result: Result<
			Vec<(String, String, String, String, String, String, String)>,
			mysql::Error,
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
