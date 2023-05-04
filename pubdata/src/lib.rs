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
	pub emoji: String,
}

#[derive(Deserialize)]
pub struct PublicData {
	pub shifts: Vec<PubDataShift>,
	pub locations: Vec<PubDataLocation>,
}

impl PublicData {
	pub fn get_location_by_name(&self, name: &String) -> Option<&PubDataLocation> {
		self.locations
			.iter()
			.find(|&location| &location.name == name)
	}

	pub fn get_shift_by_name(&self, name: &String) -> Option<&PubDataShift> {
		self.shifts.iter().find(|&shift| &shift.name == name)
	}
}
