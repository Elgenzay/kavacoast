use crate::{generic::Environment, routes};
use rocket::{
	fs::{relative, NamedFile},
	response::Redirect,
	serde::json::Json,
	shield::{Hsts, Shield},
	time::Duration,
};
use serde::Serialize;
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

#[derive(Serialize)]
pub struct VersionInfo {
	version: String,
}

#[rocket::get("/version")]
pub fn version() -> Json<VersionInfo> {
	Json(VersionInfo {
		version: env!("CARGO_PKG_VERSION").to_string(),
	})
}

pub async fn start_web() {
	if let Err(e) = rocket::build()
		.mount(
			"/",
			rocket::routes![
				static_pages,
				version,
				join,
				routes::token::token_json,
				routes::token::token_form,
				routes::check_token::check_token,
				routes::users::register,
				routes::check_registration_key::check_registration_key,
				routes::pages::dashboard::dashboard,
				routes::pages::admin::admin,
				routes::pages::settings::settings,
				routes::pages::pool_host::pool_host,
				routes::pages::pool::pool,
				routes::users::change_password,
				routes::users::get_users,
				routes::users::update_user,
				routes::users::delete_user,
				routes::users::create_referral,
				routes::users::delete_referral,
				routes::pool_player::create_pool_player,
				routes::pool_player::get_pool_players,
				routes::pool_player::get_pool_player,
				routes::pool_player::update_pool_player,
			],
		)
		.attach(Shield::default().enable(Hsts::IncludeSubDomains(Duration::new(31536000, 0))))
		.launch()
		.await
	{
		log::error!("Error starting web server: {}", e);
	}
}
