use kava_mysql::get_mysql_connection;
use mysql::prelude::Queryable;
use serde_json::json;

#[derive(Clone)]
pub struct Logger {
	guild_id: String,
	ch_id_error: String,
	ch_id_generic: String,
	ch_id_schedule: String,
}

impl Logger {
	pub fn new() -> Logger {
		dotenvy::dotenv().ok();
		Logger {
			guild_id: std::env::var("DISCORD_GUILD_ID")
				.expect("Missing environment variable: DISCORD_GUILD_ID"),
			ch_id_error: std::env::var("DISCORD_ERROR_CHANNEL_ID")
				.expect("Missing environment variable: DISCORD_ERROR_CHANNEL_ID"),
			ch_id_generic: std::env::var("DISCORD_LOG_CHANNEL_ID")
				.expect("Missing environment variable: DISCORD_LOG_CHANNEL_ID"),
			ch_id_schedule: std::env::var("DISCORD_SCHEDULE_CHANNEL_ID")
				.expect("Missing environment variable: DISCORD_SCHEDULE_CHANNEL_ID"),
		}
	}

	pub fn panic(&self, msg: String) -> ! {
		self.log_error(msg.to_string());
		panic!("{}", msg);
	}

	pub fn log_error(&self, msg: String) {
		self.log(msg, &self.guild_id, &self.ch_id_error, vec![])
	}

	pub fn log_message(&self, msg: String) {
		self.log(msg, &self.guild_id, &self.ch_id_generic, vec![])
	}

	pub fn log_schedule(&self, msg: String, reactions: Vec<&String>) {
		self.log(msg, &self.guild_id, &self.ch_id_schedule, reactions)
	}

	fn log(&self, msg: String, guild_id: &String, ch_id: &String, reactions: Vec<&String>) {
		println!("{}", msg);
		if let Err(e) = get_mysql_connection().unwrap().exec_drop(
			"INSERT INTO log_queue (guild_id, ch_id, msg, reactions) VALUES (?,?,?,?)",
			(guild_id, ch_id, msg, json!(reactions)),
		) {
			println!("Insert error (nonfatal): {}", e);
		}
	}
}

impl Default for Logger {
	fn default() -> Self {
		Self::new()
	}
}
