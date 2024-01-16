use crate::dbrecord::DBRecord;
use crate::error::ErrorResponse;
use crate::models::user::User;
use crate::routes::token::token;
use crate::routes::token::TokenRequest;
use crate::routes::token::TokenResponse;
use rocket::{
	response::status,
	serde::{json::Json, Deserialize},
};

#[derive(Debug, Deserialize)]
pub struct RegistrationRequest {
	pub username: String,
	pub display_name: String,
	pub password: String,
}

#[rocket::post("/api/register_user", format = "json", data = "<registration>")]
pub async fn register(
	registration: Json<RegistrationRequest>,
) -> Result<Json<TokenResponse>, status::Custom<Json<ErrorResponse>>> {
	let registration = registration.into_inner();
	let user = User::new(&registration).await?;
	user.db_create().await?;

	// Log them in
	let token_request = TokenRequest::new_password_grant(&user.username, &registration.password);

	token(token_request).await
}

/*

use crate::generic::BearerToken;

#[rocket::get("/api/users/me")]
pub async fn get_self(
	bearer_token: BearerToken,
) -> Result<Json<User>, status::Custom<Json<ErrorResponse>>> {
	let session = bearer_token.validate().await?;
	let user = session.user.object().await?;
	Ok(Json(user))
}

*/
