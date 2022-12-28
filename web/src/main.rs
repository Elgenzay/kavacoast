mod schedule;

use std::path::{Path, PathBuf};
use std::result::Result;

use mysql::prelude::*;
use mysql::*;

use rocket::fs::{relative, NamedFile};
use rocket::http::Status;
use rocket::response::content::RawJson;
use rocket::response::status;
use rocket::response::Redirect;
use rocket::serde::{json::Json, Deserialize};
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
			rocket::routes![
				static_pages,
				join,
				auth,
				change_password,
				schedule::schedule_post
			],
		)
		.attach(Shield::default().enable(Hsts::IncludeSubDomains(Duration::new(31536000, 0))))
}
