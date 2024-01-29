use crate::dbrecord::DBRecord;
use crate::error::Error;
use crate::error::ErrorResponse;
use crate::generic::BearerToken;
use crate::generic::GenericOkResponse;
use crate::models::session::Session;
use crate::models::user::Role;
use crate::models::user::User;
use crate::routes::token::token;
use crate::routes::token::TokenRequest;
use crate::routes::token::TokenResponse;
use rocket::http::Status;
use rocket::{
	response::status,
	serde::{json::Json, Deserialize},
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
	let mut user = get_user(id, session).await?;

	if let Some(username) = &request.username {
		let username = User::validate_username_requirements(username)?;
		user.db_update_field("username", &username).await?;
		user.username = username;
	}

	if let Some(display_name) = &request.display_name {
		let display_name = User::validate_displayname_requirements(display_name)?;
		user.db_update_field("display_name", &display_name).await?;
		user.display_name = display_name;
	}

	Ok(Json(GenericOkResponse::new()))
}
