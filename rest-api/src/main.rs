use rocket::fs::{relative, FileServer};

mod manual {
	use rocket::fs::NamedFile;
	use std::path::{Path, PathBuf};

	#[rocket::get("/<path..>")]
	pub async fn second(path: PathBuf) -> Option<NamedFile> {
		let mut path = Path::new(super::relative!("static")).join(path);
		if path.is_dir() {
			path.push("index.html");
		}
		NamedFile::open(path).await.ok()
	}
}
#[rocket::get("/api")]
fn index() -> &'static str {
	"!test"
}

#[rocket::launch]
fn rocket() -> _ {
	rocket::build()
		.mount("/", rocket::routes![manual::second])
		.mount("/", rocket::routes![index])
}
