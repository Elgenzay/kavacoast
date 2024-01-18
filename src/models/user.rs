use crate::dbrecord::DBRecord;
use crate::error::Error;
use crate::generic::Expirable;
use crate::generic::HashedString;
use crate::generic::UUID;
use crate::models::registration::Registration;
use crate::models::session::Session;
use crate::routes::users::RegistrationRequest;
use chrono::DateTime;
use chrono::Utc;
use rocket::http::Status;
use serde::Deserialize;
use serde::Serialize;

const NAME_MIN_LENGTH: usize = 2;
const NAME_MAX_LENGTH: usize = 32;

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct User {
	pub id: UUID<User>,
	pub username: String,
	pub display_name: String,
	pub password_hash: HashedString,
	pub discord_id: Option<u64>,
	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}

impl DBRecord for User {
	fn table() -> &'static str {
		"users"
	}

	fn uuid(&self) -> UUID<Self> {
		self.id.to_owned()
	}
}

impl User {
	/// Create a new User and persist it to the database.
	///
	/// The associated Registration will be deleted.
	///
	/// Default values are specified here.
	pub async fn register(registration_request: &RegistrationRequest) -> Result<Self, Error> {
		let username = Self::validate_username_requirements(&registration_request.username)?;

		if User::db_search_one("username", &username).await?.is_some() {
			return Err(Error::new(
				Status::BadRequest,
				"Username unavailable.",
				None,
			));
		};

		let registration =
			Registration::db_search_one("registration_key", &registration_request.registration_key)
				.await?
				.ok_or_else(|| {
					Error::new(Status::Unauthorized, "Invalid registration key", None)
				})?;

		registration.db_delete().await?;

		if let Some(discord_id) = registration.discord_id {
			if User::db_search_one("discord_id", &discord_id)
				.await?
				.is_some()
			{
				return Err(Error::new(
					Status::BadRequest,
					"A user with this Discord ID already exists.", // This shouldn't actually happen, but just in case.
					None,
				));
			};
		}

		let user = Self {
			id: UUID::new(User::table()),
			username,
			display_name: Self::validate_displayname_requirements(
				&registration_request.display_name,
			)?,
			password_hash: HashedString::new(&registration_request.password)?,
			discord_id: registration.discord_id,
			created_at: Utc::now(),
			updated_at: Utc::now(),
		};

		user.db_create().await?;

		Ok(user)
	}

	pub fn verify_password(&self, password: &str) -> Result<(), Error> {
		if self.password_hash.verify(password)? {
			Ok(())
		} else {
			Err(Error::generic_401())
		}
	}

	pub async fn get_session_from_refresh_token(
		&self,
		refresh_token: &str,
	) -> Result<Option<Session>, Error> {
		let sessions: Vec<Session> = Session::db_search("user", &self.id).await?;

		for session in sessions {
			if session.refresh_token_hash.verify(refresh_token)? {
				if !session.is_expired()? {
					return Ok(Some(session));
				} else {
					session.db_delete().await?;
					continue;
				}
			}
		}

		Ok(None)
	}

	fn validate_username_requirements(username: &str) -> Result<String, Error> {
		Self::validate_name_length(username)?;

		if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
			return Err(Error::new(
				Status::BadRequest,
				"Username must contain only alphanumeric characters and underscores.",
				None,
			));
		}

		Ok(username.to_lowercase())
	}

	fn validate_displayname_requirements(displayname: &str) -> Result<String, Error> {
		Self::validate_name_length(displayname)?;
		Ok(displayname.trim().to_owned())
	}

	fn validate_name_length(name: &str) -> Result<(), Error> {
		if name.len() < NAME_MIN_LENGTH {
			return Err(Error::new(
				Status::BadRequest,
				&format!("Name must be at least {} characters long.", NAME_MIN_LENGTH),
				None,
			));
		}

		if name.len() > NAME_MAX_LENGTH {
			return Err(Error::new(
				Status::BadRequest,
				&format!("Name must be at most {} characters long.", NAME_MAX_LENGTH),
				None,
			));
		}

		Ok(())
	}
}
