use crate::generic::BearerToken;
use crate::{error::ErrorResponse, generic::get_discord_username};
use rocket::{response::status, serde::json::Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct SettingsPageResponse {
	username: String,
	display_name: String,
	discord_username: Option<String>,
	discord_id: Option<String>,
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
		discord_username: if let Some(discord_id) = &user.discord_id {
			get_discord_username(discord_id).await.ok()
		} else {
			None
		},
		discord_id: user.discord_id,
	}))
}
