use crate::{dbrecord::DBRecord, generic::UUID, models::user::User};
use chrono::{DateTime, Utc};
use either::Either;
use serde::{Deserialize, Serialize};

const KEY_LENGTH: usize = 16;

#[derive(Serialize, Deserialize)]
/// A registration link that has not yet been used.
pub struct Registration {
	id: UUID<Registration>,
	created_at: DateTime<Utc>,
	updated_at: DateTime<Utc>,
	pub registration_key: String,
	/// Left if referral, or Right (discord id) if generated by the bot.
	pub referrer_or_discord: Either<UUID<User>, String>,
}

impl Default for Registration {
	fn default() -> Self {
		Self {
			id: UUID::default(),
			created_at: Utc::now(),
			updated_at: Utc::now(),
			registration_key: String::new(),
			referrer_or_discord: Either::Right(String::new()),
		}
	}
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
	pub fn from_discord_id(discord_id: &str) -> Self {
		Self {
			id: UUID::new(),
			created_at: Utc::now(),
			updated_at: Utc::now(),
			registration_key: crate::generic::random_alphanumeric_string(KEY_LENGTH),
			referrer_or_discord: Either::Right(discord_id.to_owned()),
		}
	}

	pub fn from_user(user: &User) -> Self {
		Self {
			id: UUID::new(),
			created_at: Utc::now(),
			updated_at: Utc::now(),
			registration_key: crate::generic::random_alphanumeric_string(KEY_LENGTH),
			referrer_or_discord: Either::Left(user.uuid()),
		}
	}

	pub fn dm_string(&self) -> String {
		format!(
			"Register using this link: \nhttps://kavacoast.com/register?k={}\n\nThis registration url is linked to your Discord account, and can only be used to register once.",
			self.registration_key
		)
	}
}
