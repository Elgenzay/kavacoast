use std::path::{Path, PathBuf};

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

#[rocket::post("/api/auth", format = "application/json", data = "<input_creds>")]
fn auth(input_creds: Json<Credentials>) -> &'static str {
	if input_creds.username == "test" {
		"Y"
	} else {
		"N"
	}
}

#[rocket::launch]
fn rocket() -> _ {
	rocket::build()
		.mount("/", rocket::routes![static_pages])
		.mount("/", rocket::routes![api])
		.mount("/", rocket::routes![auth])
}
