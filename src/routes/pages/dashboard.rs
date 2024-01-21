use crate::error::ErrorResponse;
use crate::generic::BearerToken;
use crate::models::user::Role;
use rocket::{response::status, serde::json::Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct DashboardResponse {
	display_name: String,
	is_admin: bool,
}

#[rocket::get("/api/page/dashboard")]
pub async fn dashboard(
	bearer_token: BearerToken,
) -> Result<Json<DashboardResponse>, status::Custom<Json<ErrorResponse>>> {
	let session = bearer_token.validate().await?;
	let user = session.user.object().await?;

	Ok(Json(DashboardResponse {
		display_name: user.display_name.to_owned(),
		is_admin: user.has_role(&Role::Admin),
	}))
}
