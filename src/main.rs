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
	jobs::Job::spawn_all();

	tokio::spawn(async {
		kavabot::start_bot().await;
	});

	web::start_web().await;
	log::info!("Shutting down...");
}
