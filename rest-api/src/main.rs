use rocket::fs::relative;
use rocket::fs::NamedFile;
use std::path::{Path, PathBuf};

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

#[rocket::launch]
fn rocket() -> _ {
	rocket::build()
		.mount("/", rocket::routes![static_pages])
		.mount("/", rocket::routes![api])
}
