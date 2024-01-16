use generic::Environment;
use rocket::response::Redirect;
use std::path::PathBuf;

#[rocket::get("/<path..>")]
pub async fn redirect(path: PathBuf) -> Redirect {
	let mut new_uri = String::from("https://");
	let domain = Environment::new().domain.val();
	new_uri.push_str(&domain);
	new_uri.push_str(&path.into_os_string().into_string().unwrap());
	Redirect::to(new_uri)
}

#[rocket::launch]
fn rocket() -> _ {
	Environment::load_path("config.toml");
	rocket::build().mount("/", rocket::routes![redirect])
}
