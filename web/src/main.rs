use std::path::{Path, PathBuf};
use std::result::Result;

use mysql::prelude::*;
use mysql::*;

use rocket::fs::{relative, NamedFile};
use rocket::http::Status;
use rocket::response::content::RawJson;
use rocket::response::status;
use rocket::response::Redirect;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::shield::Hsts;
use rocket::shield::Shield;
use rocket::time::Duration;

use argon2::{
	password_hash::{
		rand_core::OsRng, PasswordHashString, PasswordHasher, PasswordVerifier, SaltString,
	},
	Argon2,
};

#[rocket::get("/<path..>")]
pub async fn static_pages(path: PathBuf) -> Option<NamedFile> {
	let mut path = Path::new(relative!("static")).join(path);
	if path.is_dir() {
		path.push("index.html");
	}
	NamedFile::open(path).await.ok()
}

#[rocket::get("/join")]
pub async fn join() -> Redirect {
	let inv = std::env::var("DISCORD_INVITE_LINK")
		.expect("Missing environment variable: DISCORD_INVITE_LINK");
	Redirect::to(inv)
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct Credentials {
	username: String,
	password: String,
}

impl Credentials {
	fn authenticate(&self) -> Result<Bartender, (Status, String)> {
		let bartender = Bartender::find(&self.username)?;
		if bartender.hash == "0" {
			return Ok(bartender);
		}
		let passwordhashstring = match PasswordHashString::new(&bartender.hash) {
			Ok(v) => v,
			Err(e) => return Err((Status::InternalServerError, e.to_string())),
		};
		let passwordhash = passwordhashstring.password_hash();
		let matches = Argon2::default().verify_password(&self.password.as_bytes(), &passwordhash);
		match matches {
			Ok(_) => Ok(bartender),
			Err(_) => Err((Status::Unauthorized, "Credentials invalid".to_string())),
		}
	}
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct ChangePassRequest {
	verify: Credentials,
	new_pass: String,
}

struct Bartender {
	name: String,
	hash: String,
}

impl Bartender {
	fn find(name: &String) -> Result<Bartender, (Status, String)> {
		let bartenders = get_bartenders()?;
		for bt in bartenders {
			if name == &bt.name {
				return Ok(bt);
			}
		}
		Err((
			Status::from_code(401).unwrap(),
			"Credentials invalid".to_owned(),
		))
	}

	fn update_hash(&self, hash: &str) -> Result<(), (Status, String)> {
		let mut conn = get_mysql_connection()?;
		let result: Result<Vec<_>, mysql::Error> = conn.exec::<String, &str, (&str, &str)>(
			&"UPDATE kava.`bartenders` SET `hash`=? WHERE `name`=?;".to_owned(),
			(hash, &self.name[..]),
		);
		match result {
			Ok(_) => Ok(()),
			Err(e) => Err((Status::InternalServerError, format!("MySQL error: {}", e))),
		}
	}
}

#[rocket::post(
	"/api/change_password",
	format = "application/json",
	data = "<request>"
)]
fn change_password(request: Json<ChangePassRequest>) -> status::Custom<RawJson<String>> {
	let result = |request: Json<ChangePassRequest>| -> Result<(), (Status, String)> {
		let bartender = request.verify.authenticate()?;
		let salt = SaltString::generate(&mut OsRng);
		let p_bytes = request.new_pass.as_bytes();
		let password_hash = match Argon2::default().hash_password(p_bytes, &salt) {
			Ok(v) => v,
			Err(e) => return Err((Status::InternalServerError, e.to_string())),
		};
		bartender.update_hash(&password_hash.serialize().to_string())
	};
	match result(request) {
		Ok(_) => status::Custom(Status::Ok, RawJson("{\"success\":true}".to_owned())),
		Err(e) => status::Custom(e.0, RawJson(format!("{{\"error\":\"{}\"}}", e.1))),
	}
}

#[rocket::post("/api/auth", format = "application/json", data = "<input_creds>")]
fn auth(input_creds: Json<Credentials>) -> status::Custom<RawJson<String>> {
	match input_creds.authenticate() {
		Ok(_) => status::Custom(Status::Ok, RawJson("{\"success\":true}".to_string())),
		Err(e) => status::Custom(e.0, RawJson(format!("{{\"error\":\"{}\"}}", e.1))),
	}
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct Schedule {
	week1: ScheduleWeek,
	week2: ScheduleWeek,
}

impl Schedule {
	fn get_week(&self, number: i8) -> &ScheduleWeek {
		match number {
			1 => &self.week1,
			2 => &self.week2,
			_ => panic!("Invalid week"),
		}
	}
}

#[derive(Deserialize)]
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
	fn get(&self, day: &str) -> &ScheduleDay {
		match day {
			"sun" => &self.sun,
			"mon" => &self.mon,
			"tue" => &self.tue,
			"wed" => &self.wed,
			"thu" => &self.thu,
			"fri" => &self.fri,
			"sat" => &self.sat,
			_ => panic!("Invalid day"),
		}
	}
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct ScheduleDay {
	locations: Vec<ScheduleLocation>,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct ScheduleLocation {
	name: String,
	shifts: Vec<ScheduleShift>,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct ScheduleShift {
	name: String,
	bartender: String,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct SchedulePostRequest {
	verify: Credentials,
	schedule: Schedule,
}

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
			_ => panic!("Invalid day"),
		};
	}
}

#[rocket::post("/api/schedule", format = "application/json", data = "<request>")]
fn schedule_post(request: Json<SchedulePostRequest>) -> status::Custom<RawJson<String>> {
	let result = |request: Json<SchedulePostRequest>| -> Result<(), (Status, String)> {
		request.verify.authenticate()?;
		let mut conn = get_mysql_connection()?;
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
				for day_i in vec!["sun", "mon", "tue", "wed", "thu", "fri", "sat"] {
					for day_loc in &request.schedule.get_week(week_i).get(day_i).locations {
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

fn get_bartenders() -> Result<Vec<Bartender>, (Status, String)> {
	let mut conn = get_mysql_connection()?;
	let selected_bartenders_result = conn
		.query_map("SELECT name, hash from bartenders", |(name, hash)| {
			Bartender { name, hash }
		});
	match selected_bartenders_result {
		Ok(v) => Ok(v),
		Err(e) => Err((Status::InternalServerError, e.to_string())),
	}
}

fn get_mysql_connection() -> Result<PooledConn, (Status, String)> {
	let pass = std::env::var("MYSQL_PASS").expect("Missing environment variable: MYSQL_PASS");
	let url: &str =
		&(String::from("mysql://kava:") + &pass + &String::from("@localhost:3306/kava"))[..];
	let pool = match Pool::new(url) {
		Ok(v) => v,
		Err(e) => return Err((Status::InternalServerError, e.to_string())),
	};
	match pool.get_conn() {
		Ok(v) => Ok(v),
		Err(e) => Err((Status::InternalServerError, e.to_string())),
	}
}

#[rocket::launch]
fn rocket() -> _ {
	dotenvy::dotenv().ok();
	rocket::build()
		.mount(
			"/",
			rocket::routes![static_pages, join, auth, change_password, schedule_post],
		)
		.attach(Shield::default().enable(Hsts::IncludeSubDomains(Duration::new(31536000, 0))))
}
