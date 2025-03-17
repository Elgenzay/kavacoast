use crate::{
	dbrecord::DBRecord,
	error::{Error, ErrorResponse},
	generic::{BearerToken, GenericOkResponse, UUID},
	models::{
		pool_game::{PoolGame, PoolGameType, PoolGameWinner},
		pool_player::PoolPlayer,
		user::User,
	},
	routes::pool_player::require_pool_host,
};
use chrono::NaiveDate;
use core::str;
use rocket::{http::Status, response::status, serde::json::Json};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct UpdatePoolGameRequest {
	date: Option<String>,
	player1: Option<UUID<PoolPlayer>>,
	player2: Option<UUID<PoolPlayer>>,
	winner: Option<PoolGameWinner>,
	host: Option<UUID<User>>,
	game_type: Option<PoolGameType>,
}

#[rocket::post("/api/pool_games", format = "json", data = "<request>")]
pub async fn create_pool_game(
	request: Json<UpdatePoolGameRequest>,
	bearer_token: BearerToken,
) -> Result<Json<PoolGame>, status::Custom<Json<ErrorResponse>>> {
	let session = bearer_token.validate().await?;
	require_pool_host(&session).await?;

	let (player1, player2) = match (request.player1.as_ref(), request.player2.as_ref()) {
		(Some(player1), Some(player2)) => (player1, player2),
		_ => return Err(Error::new(Status::BadRequest, "Missing required field(s)", None).into()),
	};

	let player1 = match PoolPlayer::db_by_id(&player1.uuid_string()).await? {
		Some(player) => player,
		None => return Err(Error::new(Status::NotFound, "Player 1 not found", None).into()),
	};

	let player2 = match PoolPlayer::db_by_id(&player2.uuid_string()).await? {
		Some(player) => player,
		None => return Err(Error::new(Status::NotFound, "Player 2 not found", None).into()),
	};

	let game = PoolGame::new(player1.uuid(), player2.uuid(), session.user().await?.uuid());
	game.db_create().await?;
	Ok(Json(game))
}

#[rocket::patch("/api/pool_games/<id>", format = "json", data = "<request>")]
pub async fn update_pool_game(
	id: String,
	request: Json<UpdatePoolGameRequest>,
	bearer_token: BearerToken,
) -> Result<Json<GenericOkResponse>, status::Custom<Json<ErrorResponse>>> {
	let session = bearer_token.validate().await?;
	require_pool_host(&session).await?;

	let mut updates = vec![];

	let game = PoolGame::db_by_id(&id)
		.await?
		.ok_or_else(|| Error::new(Status::NotFound, "Pool game not found", None))?;

	if let Some(date) = &request.date {
		let naivedate = NaiveDate::parse_from_str(date, "%Y-%m-%d")
			.map_err(|_| Error::new(Status::BadRequest, "Invalid date format", None))?;

		updates.push(("date", json!(naivedate)));
	}

	if let Some(player) = &request.player1 {
		if let Some(player) = PoolPlayer::db_by_id(&player.uuid_string()).await? {
			updates.push(("player1", json!(player.uuid())));
		} else {
			return Err(Error::new(Status::NotFound, "Player 1 not found", None).into());
		}
	}

	if let Some(player) = &request.player2 {
		if let Some(player) = PoolPlayer::db_by_id(&player.uuid_string()).await? {
			updates.push(("player2", json!(player.uuid())));
		} else {
			return Err(Error::new(Status::NotFound, "Player 2 not found", None).into());
		}
	}

	if let Some(winner) = &request.winner {
		updates.push(("winner", json!(winner)));
	}

	if let Some(game_type) = &request.game_type {
		updates.push(("game_type", json!(game_type)));
	}

	if let Some(host) = &request.host {
		if let Some(host) = User::db_by_id(&host.uuid_string()).await? {
			updates.push(("host", json!(host.uuid())));
		} else {
			return Err(Error::new(Status::NotFound, "Host user not found", None).into());
		}
	}

	game.db_update_fields(updates).await?;

	Ok(Json(GenericOkResponse::new()))
}

#[rocket::get("/api/pool_games/<id>")]
pub async fn get_pool_game(
	id: String,
	bearer_token: BearerToken,
) -> Result<Json<PoolGame>, status::Custom<Json<ErrorResponse>>> {
	let session = bearer_token.validate().await?;
	require_pool_host(&session).await?;

	let game = PoolGame::db_by_id(&id)
		.await?
		.ok_or_else(|| Error::new(Status::NotFound, "Pool game not found", None))?;

	Ok(Json(game))
}

#[rocket::get("/api/pool_games")]
pub async fn get_pool_games(
	bearer_token: BearerToken,
) -> Result<Json<Vec<PoolGame>>, status::Custom<Json<ErrorResponse>>> {
	let session = bearer_token.validate().await?;
	require_pool_host(&session).await?;
	Ok(Json(PoolGame::db_all().await?))
}

#[rocket::delete("/api/pool_games/<id>")]
pub async fn delete_pool_game(
	id: String,
	bearer_token: BearerToken,
) -> Result<Json<GenericOkResponse>, status::Custom<Json<ErrorResponse>>> {
	let session = bearer_token.validate().await?;
	require_pool_host(&session).await?;

	let game = PoolGame::db_by_id(&id)
		.await?
		.ok_or_else(|| Error::new(Status::NotFound, "Pool game not found", None))?;

	game.db_delete().await?;
	Ok(Json(GenericOkResponse::new()))
}
