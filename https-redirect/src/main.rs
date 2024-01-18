use rocket::response::Redirect;
use std::path::PathBuf;

#[rocket::get("/<path..>")]
pub async fn redirect(path: PathBuf) -> Redirect {
	let new_uri = format!(
		"https://kavacoast.com/{}",
		path.into_os_string().into_string().unwrap()
	);
	Redirect::to(new_uri)
}

#[rocket::launch]
fn rocket() -> _ {
	rocket::build().mount("/", rocket::routes![redirect])
}
