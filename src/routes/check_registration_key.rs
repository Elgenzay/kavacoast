use crate::{
	dbrecord::DBRecord,
	error::{Error, ErrorResponse},
	generic::GenericOkResponse,
	models::registration::Registration,
};
use rocket::{
	http::Status,
	response::status,
	serde::{json::Json, Deserialize},
};

#[derive(Deserialize)]
pub struct CheckRegistrationKeyRequest {
	registration_key: String,
}

#[rocket::post("/api/check_registration_key", format = "json", data = "<request>")]
pub async fn check_registration_key(
	request: Json<CheckRegistrationKeyRequest>,
) -> Result<Json<GenericOkResponse>, status::Custom<Json<ErrorResponse>>> {
	Registration::db_search_one("registration_key", request.registration_key.clone())
		.await?
		.ok_or_else(|| Error::new(Status::Unauthorized, "Invalid registration key", None))?;

	Ok(Json(GenericOkResponse::new()))
}
