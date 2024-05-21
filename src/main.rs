use dbrecord::DBRecord;
use generic::Environment;
use std::env;

mod cmds;
mod dbrecord;
mod error;
mod generic;
mod jobs;
mod kavabot;
mod models;
mod routes;
mod web;

#[tokio::main]
async fn main() {
	env_logger::builder()
		.filter_level(log::LevelFilter::Info)
		.filter_module("tracing::span", log::LevelFilter::Warn)
		.filter_module("serenity", log::LevelFilter::Warn)
		.init();

	log::info!("Starting...");
	generic::Environment::load_path("config.toml");
	let args: Vec<String> = env::args().collect();

	if args.contains(&"test".to_string()) {
		test_init().await;
		return;
	}

	jobs::Job::spawn_all();

	tokio::spawn(async {
		kavabot::start_bot().await;
	});

	web::start_web().await;
	log::info!("Shutting down...");
}

async fn test_init() {
	log::info!("Initializing test environment");
	models::user::User::db_delete_table().await.unwrap();

	let mut admin = models::user::User {
		username: "admin".to_owned(),
		display_name: "Admin".to_owned(),
		discord_id: Some(Environment::new().admin_id.val()),
		..Default::default()
	};

	admin.db_create().await.unwrap();
	admin.set_password("admin123").await.unwrap();
}
