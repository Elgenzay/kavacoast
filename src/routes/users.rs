use crate::{
	dbrecord::DBRecord,
	error::{Error, ErrorResponse},
	generic::{BearerToken, GenericOkResponse},
	models::{
		registration::Registration,
		session::Session,
		user::{Role, User},
	},
	routes::token::{token, TokenRequest, TokenResponse},
};
use core::str;
use either::Either;
use rocket::{
	http::Status,
	response::status,
	serde::{json::Json, Deserialize, Serialize},
};
use surrealdb::sql::Id;

#[derive(Deserialize)]
pub struct RegistrationRequest {
	pub username: String,
	pub display_name: String,
	pub password: String,
	pub registration_key: String,
}

#[rocket::post("/api/register_user", format = "json", data = "<registration>")]
pub async fn register(
	registration: Json<RegistrationRequest>,
) -> Result<Json<TokenResponse>, status::Custom<Json<ErrorResponse>>> {
	let registration = registration.into_inner();
	let user = User::register(&registration).await?;

	// Log them in
	let token_request = TokenRequest::new_password_grant(&user.username, &registration.password);

	token(token_request).await
}

/// Retrieves a user by their ID, subject to security checks based on the session.
///
/// This function accepts a user ID and a session. If the ID is "me", it returns the session's user.
/// Otherwise, it returns the user corresponding to the provided ID only if the session's user is
/// the same as the user with the ID or if the session's user is an admin.
async fn get_user(id: &str, session: Session) -> Result<User, Error> {
	if id == "me" {
		session.user.object().await
	} else {
		match User::db_by_id(Id::from(id)).await? {
			Some(user) => {
				if user.id != session.user && !session.user.object().await?.has_role(&Role::Admin) {
					return Err(Error::new(
						Status::Unauthorized,
						"Insufficient permissions",
						None,
					));
				}
				Ok(user)
			}
			None => Err(Error::new(Status::NotFound, "User not found", None)),
		}
	}
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
	pub old_password: String,
	pub new_password: String,
}

#[rocket::post("/api/users/<id>/change_password", format = "json", data = "<request>")]
pub async fn change_password(
	id: String,
	request: Json<ChangePasswordRequest>,
	bearer_token: BearerToken,
) -> Result<Json<GenericOkResponse>, status::Custom<Json<ErrorResponse>>> {
	let session = bearer_token.validate().await?;
	let mut user = get_user(&id, session).await?;

	if user.verify_password(&request.old_password).is_err() {
		return Err(Error::new(Status::Unauthorized, "Invalid password", None).into());
	}

	user.set_password(&request.new_password).await?;

	Ok(Json(GenericOkResponse::new()))
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
	pub username: Option<String>,
	pub display_name: Option<String>,
}

#[rocket::patch("/api/users/<id>", format = "json", data = "<request>")]
pub async fn update_user(
	id: &str,
	request: Json<UpdateUserRequest>,
	bearer_token: BearerToken,
) -> Result<Json<GenericOkResponse>, status::Custom<Json<ErrorResponse>>> {
	let session = bearer_token.validate().await?;
	let user = get_user(id, session).await?;

	if let Some(username) = &request.username {
		let username = User::validate_username_requirements(username)?;
		user.db_update_field("username", &username).await?;
	}

	if let Some(display_name) = &request.display_name {
		let display_name = User::validate_displayname_requirements(display_name)?;
		user.db_update_field("display_name", &display_name).await?;
	}

	Ok(Json(GenericOkResponse::new()))
}

#[derive(Serialize, Deserialize)]
pub struct RequestReferral {
	pub key: String,
}

#[rocket::post("/api/users/<id>/referrals", format = "json")]
pub async fn create_referral(
	id: &str,
	bearer_token: BearerToken,
) -> Result<Json<RequestReferral>, status::Custom<Json<ErrorResponse>>> {
	let session = bearer_token.validate().await?;
	let user = get_user(id, session).await?;

	if user.referral_registrations.len() >= 5 {
		return Err(Error::new(Status::BadRequest, "Too many referral links active", None).into());
	}

	if user.discord_id.is_none() {
		return Err(Error::new(
			Status::BadRequest,
			"User must have a Discord account linked to create a referral link",
			None,
		)
		.into());
	}

	let registration = Registration::from_user(&user).db_create().await?;

	Ok(Json(RequestReferral {
		key: registration.registration_key,
	}))
}

#[rocket::delete("/api/users/<id>/referrals", format = "json", data = "<request>")]
pub async fn delete_referral(
	id: &str,
	request: Json<RequestReferral>,
	bearer_token: BearerToken,
) -> Result<Json<GenericOkResponse>, status::Custom<Json<ErrorResponse>>> {
	let session = bearer_token.validate().await?;
	let user: User = get_user(id, session).await?;

	let registration = Registration::db_search_one("registration_key", &request.key)
		.await?
		.ok_or_else(|| Error::new(Status::NotFound, "Referral not found", None))?;

	if let Either::Left(referred_by) = &registration.referrer_or_discord {
		if referred_by == &user.id {
			registration.db_delete().await?;
			let mut new_referrals = vec![];

			for ref_uuid in &user.referral_registrations {
				if let Some(referral) = Registration::db_by_id(ref_uuid.id()).await? {
					if referral.registration_key != request.key {
						new_referrals.push(registration.uuid());
					}
				}
			}

			user.db_update_field("referral_registrations", &new_referrals)
				.await?;

			return Ok(Json(GenericOkResponse::new()));
		}
	}

	Err(Error::new(
		Status::BadRequest,
		"Referral was created by another user",
		None,
	)
	.into())
}
