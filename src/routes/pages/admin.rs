use crate::error::{Error, ErrorResponse};
use crate::generic::BearerToken;
use crate::models::user::Role;
use rocket::{response::status, serde::json::Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct AdminPageResponse {
	//
}

#[rocket::get("/api/page/admin")]
pub async fn admin(
	bearer_token: BearerToken,
) -> Result<Json<AdminPageResponse>, status::Custom<Json<ErrorResponse>>> {
	let session = bearer_token.validate().await?;
	let user = session.user.object().await?;

	if !user.has_role(&Role::Admin) {
		return Err(Error::forbidden().into());
	}

	Ok(Json(AdminPageResponse {}))
}
