use crate::{
	dbrecord::DBRecord,
	error::{Error, ErrorResponse},
	generic::BearerToken,
	models::{
		pool_game::PoolGame,
		pool_player::PoolPlayer,
		user::{Role, User},
	},
};
use rocket::{response::status, serde::json::Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct PoolHostPageResponse {
	users: Vec<PoolHostPageUser>,
	pool_players: Vec<PoolPlayer>,
	games: Vec<PoolGame>,
}

#[derive(Serialize)]
pub struct PoolHostPageUser {
	username: String,
	display_name: String,
	id: String,
}

#[rocket::get("/api/page/pool_host")]
pub async fn pool_host(
	bearer_token: BearerToken,
) -> Result<Json<PoolHostPageResponse>, status::Custom<Json<ErrorResponse>>> {
	let session = bearer_token.validate().await?;
	let user = session.user().await?;

	if !user.has_role(&Role::PoolHost) {
		return Err(Error::forbidden().into());
	}

	let users: Vec<PoolHostPageUser> = User::db_all()
		.await?
		.into_iter()
		.map(|user| PoolHostPageUser {
			username: user.username,
			display_name: user.display_name,
			id: user.id.id().to_raw(),
		})
		.collect();

	Ok(Json(PoolHostPageResponse {
		users,
		pool_players: PoolPlayer::db_all().await?,
		games: PoolGame::db_all().await?,
	}))
}
