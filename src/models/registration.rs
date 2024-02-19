use crate::{dbrecord::DBRecord, generic::UUID};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

const KEY_LENGTH: usize = 16;

#[derive(Serialize, Deserialize, Default)]
/// A registration link that has not yet been used.
pub struct Registration {
	id: UUID<Registration>,
	created_at: DateTime<Utc>,
	updated_at: DateTime<Utc>,
	registration_key: String,
	pub discord_id: Option<String>,
}

impl DBRecord for Registration {
	fn table() -> &'static str {
		"registrations"
	}

	fn uuid(&self) -> UUID<Self> {
		self.id.to_owned()
	}
}

impl Registration {
	pub fn new(discord_id: Option<String>) -> Self {
		Self {
			id: UUID::new(),
			created_at: Utc::now(),
			updated_at: Utc::now(),
			registration_key: crate::generic::random_alphanumeric_string(KEY_LENGTH),
			discord_id,
		}
	}

	pub fn dm_string(&self) -> String {
		format!(
			"Register using this link: \nhttps://kavacoast.com/register?k={}\n\nThis registration url is linked to your Discord account, and can only be used to register once.",
			self.registration_key
		)
	}
}
