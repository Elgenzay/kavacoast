use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use surrealdb::{engine::remote::ws::Ws, opt::auth::Root, Surreal};

pub async fn surrealdb_client() -> Result<Surreal<surrealdb::engine::remote::ws::Client>, String> {
	let env = Environment::new();

	let db = Surreal::new::<Ws>(env.surreal_address.val())
		.await
		.map_err(|e| "Error connecting to SurrealDB: ".to_owned() + &e.to_string())?;

	db.signin(Root {
		username: &env.surreal_username.val(),
		password: &env.surreal_password.val(),
	})
	.await
	.map_err(|e| "Error signing in to SurrealDB: ".to_owned() + &e.to_string())?;

	db.use_ns(env.surreal_namespace.val())
		.use_db(env.surreal_database.val())
		.await
		.map_err(|e| "Error using namespace/database: ".to_owned() + &e.to_string())?;

	Ok(db)
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Environment {
	pub admin_id: EnvVarKey,
	pub bot_token: EnvVarKey,
	pub surreal_password: EnvVarKey,
	pub surreal_username: EnvVarKey,
	pub surreal_address: EnvVarKey,
	pub surreal_namespace: EnvVarKey,
	pub surreal_database: EnvVarKey,
	pub domain: EnvVarKey,
	pub discord_invite_link: EnvVarKey,
}

macro_rules! initialize_env {
    ($($field:ident),+) => {
        pub fn initialize_env(&mut self) {
            $(self.$field = EnvVarKey(stringify!($field).to_uppercase());)*
        }
    };
}

impl Environment {
	pub fn new() -> Self {
		let mut env = Self::default();
		env.initialize_env();
		env
	}

	initialize_env!(
		admin_id,
		bot_token,
		surreal_password,
		surreal_username,
		surreal_address,
		domain,
		discord_invite_link
	);

	pub fn load_path(path: &str) {
		let env: Self =
			confy::load_path(path).unwrap_or_else(|err| panic!("Failed to load config:: {}", err));

		let map = env.as_hashmap();

		for (key, value) in map.iter() {
			std::env::set_var(key, &value.0);
		}
	}

	fn as_hashmap(&self) -> HashMap<String, EnvVarKey> {
		let value = serde_json::to_value(self).unwrap();
		let mut map = HashMap::new();

		for (key, value) in value.as_object().unwrap().iter() {
			let value = value.as_str().unwrap();
			map.insert(key.to_string(), EnvVarKey(value.to_string()));
		}

		map
	}
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct EnvVarKey(String);

impl EnvVarKey {
	pub fn val(&self) -> String {
		std::env::var(&self.0)
			.unwrap_or_else(|_| panic!("Missing environment variable: {}", self.0))
	}
}
