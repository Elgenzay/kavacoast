use std::path::{Path, PathBuf};

use mysql::prelude::*;
use mysql::*;

use rocket::data::FromData;
use rocket::fs::{relative, NamedFile};
use rocket::serde::{json::Json, Deserialize};

use argon2::Argon2;
use pbkdf2::Pbkdf2;
use scrypt::Scrypt;

#[rocket::get("/<path..>")]
pub async fn static_pages(path: PathBuf) -> Option<NamedFile> {
	let mut path = Path::new(relative!("static")).join(path);
	if path.is_dir() {
		path.push("index.html");
	}
	NamedFile::open(path).await.ok()
}

#[rocket::get("/api")]
fn api() -> &'static str {
	"!test"
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct Credentials {
	username: String,
	password: String,
}

struct Bartender {
	name: String,
	hash: String,
	salt: String,
}

#[rocket::post("/api/auth", format = "application/json", data = "<input_creds>")]
fn auth(input_creds: Json<Credentials>) -> &'static str {
	let sql_result = get_bartenders();
	match sql_result {
		Ok(_) => (),
		Err(e) => {
			println!("error");
			return "error";
		}
	}
	let bartenders = sql_result.unwrap();
	for tender in bartenders {
		if input_creds.username == tender.name {
			return "match";
		}
	}

	"no match"

	/*
	if input_creds.username == "test" {
		"Y"
	} else {
		"N"
	}
	*/
}

fn get_bartenders() -> std::result::Result<Vec<Bartender>, Box<dyn std::error::Error>> {
	dotenvy::dotenv().ok();
	let pass = std::env::var("MYSQL_PASS").expect("Missing environment variable: MYSQL_PASS");
	let url: &str =
		&(String::from("mysql://mysql:") + &pass + &String::from("@localhost:3306/kava"))[..];
	let pool = Pool::new(url)?;
	let mut conn = pool.get_conn()?;
	let selected_bartenders = conn.query_map(
		"SELECT name, hash, salt from bartenders",
		|(name, hash, salt)| Bartender { name, hash, salt },
	)?;
	Ok(selected_bartenders)
}

#[rocket::launch]
fn rocket() -> _ {
	rocket::build()
		.mount("/", rocket::routes![static_pages])
		.mount("/", rocket::routes![api])
		.mount("/", rocket::routes![auth])
}
