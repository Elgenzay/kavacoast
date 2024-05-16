use crate::{
	dbrecord::DBRecord,
	error::{Error, ErrorResponse},
	generic::BearerToken,
	models::pool_player::PoolPlayer,
};
use rocket::{response::status, serde::json::Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct PoolResponse {
	player: PoolPlayer,
}

#[rocket::get("/api/page/pool")]
pub async fn pool(
	bearer_token: BearerToken,
) -> Result<Json<PoolResponse>, status::Custom<Json<ErrorResponse>>> {
	let session = bearer_token.validate().await?;
	let user = session.user.object().await?;

	let player = PoolPlayer::db_search_one("user", &user.id)
		.await?
		.ok_or_else(Error::forbidden)?;

	Ok(Json(PoolResponse { player }))
}
