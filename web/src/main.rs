extern crate generic;

use generic::Environment;
use rocket::fs::{relative, NamedFile};
use rocket::response::Redirect;
use rocket::shield::Hsts;
use rocket::shield::Shield;
use rocket::time::Duration;
use std::path::{Path, PathBuf};

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
	let inv = Environment::new().discord_invite_link.val();
	Redirect::to(inv)
}

#[rocket::launch]
fn rocket() -> _ {
	dotenvy::dotenv().ok();
	rocket::build()
		.mount("/", rocket::routes![static_pages, join])
		.attach(Shield::default().enable(Hsts::IncludeSubDomains(Duration::new(31536000, 0))))
}
