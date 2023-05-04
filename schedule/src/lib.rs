use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct Schedule {
	pub week1: ScheduleWeek,
	pub week2: ScheduleWeek,
}

impl Schedule {
	pub fn empty() -> Schedule {
		Schedule {
			week1: ScheduleWeek::empty(),
			week2: ScheduleWeek::empty(),
		}
	}

	pub fn get_week(&mut self, number: i8) -> &mut ScheduleWeek {
		match number {
			1 => &mut self.week1,
			2 => &mut self.week2,
			_ => panic!("Invalid week: {}", number),
		}
	}
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ScheduleWeek {
	pub sun: ScheduleDay,
	pub mon: ScheduleDay,
	pub tue: ScheduleDay,
	pub wed: ScheduleDay,
	pub thu: ScheduleDay,
	pub fri: ScheduleDay,
	pub sat: ScheduleDay,
}

impl ScheduleWeek {
	pub fn empty() -> ScheduleWeek {
		ScheduleWeek {
			sun: ScheduleDay::empty(),
			mon: ScheduleDay::empty(),
			tue: ScheduleDay::empty(),
			wed: ScheduleDay::empty(),
			thu: ScheduleDay::empty(),
			fri: ScheduleDay::empty(),
			sat: ScheduleDay::empty(),
		}
	}

	pub fn days() -> Vec<String> {
		vec![
			"sun".to_owned(),
			"mon".to_owned(),
			"tue".to_owned(),
			"wed".to_owned(),
			"thu".to_owned(),
			"fri".to_owned(),
			"sat".to_owned(),
		]
	}

	pub fn get_day(&mut self, day: &String) -> &mut ScheduleDay {
		match day.as_str() {
			"sun" => &mut self.sun,
			"mon" => &mut self.mon,
			"tue" => &mut self.tue,
			"wed" => &mut self.wed,
			"thu" => &mut self.thu,
			"fri" => &mut self.fri,
			"sat" => &mut self.sat,
			_ => panic!("Invalid day: {}", day),
		}
	}
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ScheduleDay {
	pub locations: Vec<ScheduleLocation>,
}

impl ScheduleDay {
	pub fn empty() -> ScheduleDay {
		ScheduleDay { locations: vec![] }
	}

	pub fn get_location_index_by_name(&mut self, name: String) -> usize {
		for (i, location) in self.locations.iter_mut().enumerate() {
			if location.name == name {
				return i;
			}
		}
		self.locations.push(ScheduleLocation {
			name,
			shifts: vec![],
		});
		self.locations.len() - 1
	}
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ScheduleLocation {
	pub name: String,
	pub shifts: Vec<ScheduleShift>,
}

impl ScheduleLocation {
	pub fn get_shift_index_by_name(&mut self, name: String) -> usize {
		for (i, shift) in self.shifts.iter_mut().enumerate() {
			if shift.name == name {
				return i;
			}
		}
		self.shifts.push(ScheduleShift {
			name,
			bartender: String::new(),
		});
		self.shifts.len() - 1
	}
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ScheduleShift {
	pub name: String,
	pub bartender: String,
}

#[derive(Clone, Serialize)]
pub struct ScheduleRow {
	pub location: String,
	pub sun1: String,
	pub mon1: String,
	pub tue1: String,
	pub wed1: String,
	pub thu1: String,
	pub fri1: String,
	pub sat1: String,
	pub sun2: String,
	pub mon2: String,
	pub tue2: String,
	pub wed2: String,
	pub thu2: String,
	pub fri2: String,
	pub sat2: String,
}

impl ScheduleRow {
	pub fn empty(location: &String) -> ScheduleRow {
		ScheduleRow {
			location: location.to_string(),
			sun1: String::new(),
			mon1: String::new(),
			tue1: String::new(),
			wed1: String::new(),
			thu1: String::new(),
			fri1: String::new(),
			sat1: String::new(),
			sun2: String::new(),
			mon2: String::new(),
			tue2: String::new(),
			wed2: String::new(),
			thu2: String::new(),
			fri2: String::new(),
			sat2: String::new(),
		}
	}

	pub fn set_day(&mut self, day: &str, val: String) {
		match day {
			"sun1" => self.sun1 = val,
			"mon1" => self.mon1 = val,
			"tue1" => self.tue1 = val,
			"wed1" => self.wed1 = val,
			"thu1" => self.thu1 = val,
			"fri1" => self.fri1 = val,
			"sat1" => self.sat1 = val,
			"sun2" => self.sun2 = val,
			"mon2" => self.mon2 = val,
			"tue2" => self.tue2 = val,
			"wed2" => self.wed2 = val,
			"thu2" => self.thu2 = val,
			"fri2" => self.fri2 = val,
			"sat2" => self.sat2 = val,
			_ => panic!("Invalid day: {}", day),
		};
	}

	pub fn get_day(&self, day: &str) -> &String {
		match day {
			"sun1" => &self.sun1,
			"mon1" => &self.mon1,
			"tue1" => &self.tue1,
			"wed1" => &self.wed1,
			"thu1" => &self.thu1,
			"fri1" => &self.fri1,
			"sat1" => &self.sat1,
			"sun2" => &self.sun2,
			"mon2" => &self.mon2,
			"tue2" => &self.tue2,
			"wed2" => &self.wed2,
			"thu2" => &self.thu2,
			"fri2" => &self.fri2,
			"sat2" => &self.sat2,
			_ => panic!("Invalid day: {}", day),
		}
	}
}
