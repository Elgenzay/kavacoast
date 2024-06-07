use crate::{
	dbrecord::DBRecord,
	error::Error,
	generic::{Expirable, HashedString, UUID},
	models::{registration::Registration, session::Session},
	routes::users::RegistrationRequest,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use either::Either::{self, Left};
use rocket::http::Status;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

const NAME_MIN_LENGTH: usize = 2;
const NAME_MAX_LENGTH: usize = 32;
const PASSWORD_MIN_LENGTH: usize = 8;

#[derive(Serialize, Deserialize)]
pub struct User {
	pub id: UUID<User>,
	pub username: String,
	pub display_name: String,
	pub password_hash: HashedString,
	pub discord_id: Option<String>,
	pub roles: Vec<Role>,
	pub referral_registrations: Vec<UUID<Registration>>,
	pub referred_users: Vec<UUID<User>>,
	pub referred_by: Option<UUID<User>>,
	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}

impl Default for User {
	fn default() -> Self {
		Self {
			id: UUID::new(),
			username: "".to_owned(),
			display_name: "".to_owned(),
			password_hash: Default::default(),
			discord_id: None,
			roles: vec![],
			referral_registrations: vec![],
			referred_users: vec![],
			referred_by: None,
			created_at: Utc::now(),
			updated_at: Utc::now(),
		}
	}
}

#[derive(Serialize, Deserialize, PartialEq, EnumIter)]
#[serde(rename_all = "snake_case")]
pub enum Role {
	Admin,
	PoolHost,
}

impl Role {
	pub fn all() -> Vec<Self> {
		Role::iter().collect()
	}
}

#[async_trait]
impl DBRecord for User {
	fn table() -> &'static str {
		"users"
	}

	fn uuid(&self) -> UUID<Self> {
		self.id.to_owned()
	}

	async fn delete_hook(&self) -> Result<(), Error> {
		for registration_uuid in self.referral_registrations.iter() {
			let registration = Registration::db_by_id(registration_uuid.id()).await?;

			if let Some(registration) = registration {
				registration.db_delete().await?;
			}
		}

		Ok(())
	}

	fn use_trash() -> bool {
		true
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

		Self::verify_password_requirements(&registration_request.password)?;

		let (referred_by, discord_id) = match registration.referrer_or_discord {
			Either::Right(discord_id) => {
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
				(None, Some(discord_id))
			}
			Either::Left(referred_by) => (Some(referred_by), None),
		};

		let user = Self {
			id: UUID::new(),
			username,
			display_name: Self::validate_displayname_requirements(
				&registration_request.display_name,
			)?,
			password_hash: HashedString::new(&registration_request.password)?,
			discord_id,
			created_at: Utc::now(),
			updated_at: Utc::now(),
			referred_by: referred_by.clone(),
			..Default::default()
		};

		if let Some(referred_by) = referred_by {
			let mut referred_by_user =
				User::db_by_id(referred_by.id()).await?.ok_or_else(|| {
					Error::new(
						Status::InternalServerError,
						"Referrer not found",
						Some(&format!("Referrer not found: {}", referred_by.id())),
					)
				})?;

			referred_by_user.referred_users.push(user.uuid());

			referred_by_user
				.db_update_field("referred_users", &referred_by_user.referred_users)
				.await?;
		}

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

	/// Get referral registrations, removing any that don't exist from the user.
	pub async fn get_referral_registrations(&self) -> Result<Vec<Registration>, Error> {
		let mut referrals = vec![];

		for referral in &self.referral_registrations {
			if let Some(registration) = Registration::db_by_id(referral.id()).await? {
				if let Left(referred_by) = &registration.referrer_or_discord {
					if referred_by == &self.id {
						referrals.push(registration);
					}
				}
			}
		}

		if referrals.len() != self.referral_registrations.len() {
			let referral_ids: Vec<UUID<Registration>> =
				referrals.iter().map(|r| r.uuid()).collect();

			self.db_update_field("referral_registrations", &referral_ids)
				.await?;
		}

		Ok(referrals)
	}
}
