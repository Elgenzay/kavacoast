use crate::error::ErrorResponse;
use crate::generic::BearerToken;
use rocket::{response::status, serde::json::Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct SettingsPageResponse {
	username: String,
	display_name: String,
}

#[rocket::get("/api/page/settings")]
pub async fn settings(
	bearer_token: BearerToken,
) -> Result<Json<SettingsPageResponse>, status::Custom<Json<ErrorResponse>>> {
	let session = bearer_token.validate().await?;
	let user = session.user.object().await?;

	Ok(Json(SettingsPageResponse {
		username: user.username.to_owned(),
		display_name: user.display_name.to_owned(),
	}))
}
