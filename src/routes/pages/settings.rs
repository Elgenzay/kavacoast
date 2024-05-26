use crate::{
	error::ErrorResponse,
	generic::{get_discord_username, BearerToken},
};
use rocket::{response::status, serde::json::Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct SettingsPageResponse {
	username: String,
	display_name: String,
	discord_username: Option<String>,
	discord_id: Option<String>,
	referrals: Vec<String>,
}

#[rocket::get("/api/page/settings")]
pub async fn settings(
	bearer_token: BearerToken,
) -> Result<Json<SettingsPageResponse>, status::Custom<Json<ErrorResponse>>> {
	let session = bearer_token.validate().await?;
	let user = session.user().await?;

	let referrals = user
		.get_referral_registrations()
		.await?
		.iter()
		.map(|r| r.registration_key.to_owned())
		.collect();

	let discord_username = if let Some(discord_id) = &user.discord_id {
		get_discord_username(discord_id).await.ok()
	} else {
		None
	};

	Ok(Json(SettingsPageResponse {
		username: user.username.to_owned(),
		display_name: user.display_name.to_owned(),
		discord_username,
		discord_id: user.discord_id,
		referrals,
	}))
}
