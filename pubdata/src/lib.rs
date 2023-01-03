use serde::Deserialize;

#[derive(Deserialize)]
pub struct PubDataShift {
	pub name: String,
	pub friendly_name: String,
	pub description: String,
}

#[derive(Deserialize)]
pub struct PubDataLocation {
	pub name: String,
	pub friendly_name: String,
}

#[derive(Deserialize)]
pub struct PublicData {
	pub shifts: Vec<PubDataShift>,
	pub locations: Vec<PubDataLocation>,
}
