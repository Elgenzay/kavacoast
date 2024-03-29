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
const PASSWORD_MIN_LENGTH: usize = 8;

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct User {
	pub id: UUID<User>,
	pub username: String,
	pub display_name: String,
	pub password_hash: HashedString,
	pub discord_id: Option<String>,
	pub roles: Vec<Role>,
	created_at: DateTime<Utc>,
	updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Role {
	Admin,
	PoolHost,
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

		if let Some(discord_id) = &registration.discord_id {
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

		Self::verify_password_requirements(&registration_request.password)?;

		let user = Self {
			id: UUID::new(),
			username,
			display_name: Self::validate_displayname_requirements(
				&registration_request.display_name,
			)?,
			password_hash: HashedString::new(&registration_request.password)?,
			discord_id: registration.discord_id,
			created_at: Utc::now(),
			updated_at: Utc::now(),
			roles: vec![],
		};

		user.db_create().await?;

		Ok(user)
	}

	pub fn has_role(&self, role: &Role) -> bool {
		self.roles.contains(role)
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

	pub fn validate_username_requirements(username: &str) -> Result<String, Error> {
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

	pub fn validate_displayname_requirements(displayname: &str) -> Result<String, Error> {
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

	fn verify_password_requirements(password: &str) -> Result<(), Error> {
		if password.len() < PASSWORD_MIN_LENGTH {
			return Err(Error::new(
				Status::BadRequest,
				&format!(
					"Password must be at least {} characters long.",
					PASSWORD_MIN_LENGTH
				),
				None,
			));
		}

		Ok(())
	}

	/// Verify password requirements, update the password, and persist it to the database.
	pub async fn set_password(&mut self, password: &str) -> Result<(), Error> {
		Self::verify_password_requirements(password)?;
		self.password_hash = HashedString::new(password)?;

		self.db_update_field("password_hash", &self.password_hash)
			.await?;

		Ok(())
	}
}
