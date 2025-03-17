use crate::{
	dbrecord::DBRecord,
	error::ErrorResponse,
	generic::BearerToken,
	models::{pool_player::PoolPlayer, user::Role},
};
use rocket::{response::status, serde::json::Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct DashboardResponse {
	display_name: String,
	roles: Vec<Role>,
	is_pool_player: bool,
}

#[rocket::get("/api/page/dashboard")]
pub async fn dashboard(
	bearer_token: BearerToken,
) -> Result<Json<DashboardResponse>, status::Custom<Json<ErrorResponse>>> {
	let session = bearer_token.validate().await?;
	let user = session.user().await?;

	Ok(Json(DashboardResponse {
		display_name: user.display_name.to_owned(),
		roles: user.roles,
		is_pool_player: PoolPlayer::db_search_one("user", user.uuid.clone())
			.await?
			.is_some(),
	}))
}
