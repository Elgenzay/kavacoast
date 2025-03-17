use crate::{dbrecord::DBRecord, generic::UUID, models::user::User};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PoolPlayer {
	pub uuid: UUID<PoolPlayer>,
	pub user: Option<UUID<User>>,
	pub descriptor: Option<String>,
	created_at: DateTime<Utc>,
	updated_at: DateTime<Utc>,
}

impl DBRecord for PoolPlayer {
	fn table() -> &'static str {
		"pool_players"
	}

	fn uuid(&self) -> UUID<Self> {
		self.uuid.to_owned()
	}

	fn use_trash() -> bool {
		true
	}
}

#[allow(dead_code)]
impl PoolPlayer {
	pub fn new() -> Self {
		Self {
			uuid: UUID::new(),
			user: None,
			descriptor: None,
			created_at: Utc::now(),
			updated_at: Utc::now(),
		}
	}

	pub fn with_user(self, user: UUID<User>) -> Self {
		Self {
			user: Some(user),
			..self
		}
	}

	pub fn with_descriptor(self, descriptor: &str) -> Self {
		Self {
			descriptor: Some(descriptor.to_owned()),
			..self
		}
	}
}
