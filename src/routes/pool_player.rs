use crate::{
	dbrecord::DBRecord,
	error::{Error, ErrorResponse},
	generic::{BearerToken, GenericOkResponse},
	models::{
		pool_player::PoolPlayer,
		session::Session,
		user::{Role, User},
	},
};
use core::str;
use rocket::{http::Status, response::status, serde::json::Json};
use serde::Deserialize;
use surrealdb::sql::Id;

pub async fn require_pool_host(session: &Session) -> Result<(), Error> {
	if !session.user().await?.has_role(&Role::PoolHost) {
		return Err(Error::insufficient_permissions());
	}

	Ok(())
}

#[derive(Deserialize)]
pub struct UpdatePoolPlayerRequest {
	pub descriptor: Option<String>,
	pub user: Option<String>,
}

#[rocket::post("/api/pool_players", format = "json", data = "<request>")]
pub async fn create_pool_player(
	request: Json<UpdatePoolPlayerRequest>,
	bearer_token: BearerToken,
) -> Result<Json<PoolPlayer>, status::Custom<Json<ErrorResponse>>> {
	let session = bearer_token.validate().await?;
	require_pool_host(&session).await?;
	let mut player = PoolPlayer::new();

	if let Some(descriptor) = &request.descriptor {
		player = player.with_descriptor(descriptor);
	}

	if let Some(user) = &request.user {
		let user = User::db_by_id(Id::from(user)).await?;

		if let Some(user) = user {
			player = player.with_user(user.uuid());
		} else {
			return Err(Error::new(Status::NotFound, "User not found", None).into());
		}
	}

	player.db_create().await?;
	Ok(Json(player))
}

#[rocket::patch("/api/pool_players/<id>", format = "json", data = "<request>")]
pub async fn update_pool_player(
	id: String,
	request: Json<UpdatePoolPlayerRequest>,
	bearer_token: BearerToken,
) -> Result<Json<GenericOkResponse>, status::Custom<Json<ErrorResponse>>> {
	let session = bearer_token.validate().await?;
	require_pool_host(&session).await?;

	let player = PoolPlayer::db_by_id(Id::from(&id))
		.await?
		.ok_or_else(|| Error::new(Status::NotFound, "Pool player not found", None))?;

	if let Some(descriptor) = &request.descriptor {
		player.db_update_field("descriptor", descriptor).await?;
	}

	if let Some(user) = &request.user {
		if let Some(user) = User::db_by_id(Id::from(user)).await? {
			player.db_update_field("user", &user.uuid()).await?;
		} else {
			return Err(Error::new(Status::NotFound, "User not found", None).into());
		}
	}

	Ok(Json(GenericOkResponse::new()))
}

#[rocket::get("/api/pool_players")]
pub async fn get_pool_players(
	bearer_token: BearerToken,
) -> Result<Json<Vec<PoolPlayer>>, status::Custom<Json<ErrorResponse>>> {
	let session = bearer_token.validate().await?;
	require_pool_host(&session).await?;
	Ok(Json(PoolPlayer::db_all().await?))
}

#[rocket::get("/api/pool_players/<id>")]
pub async fn get_pool_player(
	id: String,
	bearer_token: BearerToken,
) -> Result<Json<PoolPlayer>, status::Custom<Json<ErrorResponse>>> {
	let session = bearer_token.validate().await?;
	let user = session.user().await?;

	if id == "me" {
		return if let Some(player) = PoolPlayer::db_search_one("user", &user.uuid()).await? {
			Ok(Json(player))
		} else {
			Err(Error::new(Status::NotFound, "Pool player not found", None).into())
		};
	}

	require_pool_host(&session).await?;

	if let Some(player) = PoolPlayer::db_by_id(Id::from(id)).await? {
		Ok(Json(player))
	} else {
		Err(Error::new(Status::NotFound, "Pool player not found", None).into())
	}
}