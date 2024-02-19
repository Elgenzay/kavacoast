use crate::dbrecord::DBRecord;
use crate::generic::BearerToken;
use crate::models::user::Role;
use crate::{error::ErrorResponse, models::pool_player::PoolPlayer};
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
	let user = session.user.object().await?;

	Ok(Json(DashboardResponse {
		display_name: user.display_name.to_owned(),
		roles: user.roles,
		is_pool_player: PoolPlayer::db_search_one("user", &user.id).await?.is_some(),
	}))
}
