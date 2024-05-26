use crate::{
	dbrecord::DBRecord,
	generic::Environment,
	models::user::{Role, User},
};

pub async fn test_init() {
	log::info!("Initializing test environment");
	User::db_delete_table().await.unwrap();

	let mut admin = User {
		username: "admin".to_owned(),
		display_name: "Admin".to_owned(),
		discord_id: Some(Environment::new().admin_id.val()),
		roles: vec![Role::Admin, Role::PoolHost],
		..Default::default()
	};

	admin.db_create().await.unwrap();
	admin.set_password("admin123").await.unwrap();
}
