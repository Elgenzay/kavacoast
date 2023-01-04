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
		for location in &self.locations {
			if &location.name == name {
				return Some(location);
			}
		}
		None
	}

	pub fn get_shift_by_name(&self, name: &String) -> Option<&PubDataShift> {
		for shift in &self.shifts {
			if &shift.name == name {
				return Some(shift);
			}
		}
		None
	}
}
