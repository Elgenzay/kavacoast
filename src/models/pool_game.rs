use crate::{dbrecord::DBRecord, generic::UUID};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct PoolGame {
	pub id: UUID<PoolGame>,
	created_at: DateTime<Utc>,
	updated_at: DateTime<Utc>,
}

impl DBRecord for PoolGame {
	fn table() -> &'static str {
		"pool_games"
	}

	fn uuid(&self) -> UUID<Self> {
		self.id.to_owned()
	}
}
