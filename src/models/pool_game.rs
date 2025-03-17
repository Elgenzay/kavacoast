use crate::{
	dbrecord::DBRecord,
	generic::UUID,
	models::{pool_player::PoolPlayer, user::User},
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PoolGame {
	pub uuid: UUID<PoolGame>,
	date: NaiveDate,
	player1: UUID<PoolPlayer>,
	player2: UUID<PoolPlayer>,
	winner: PoolGameWinner,
	game_type: PoolGameType,
	host: UUID<User>,
	created_at: DateTime<Utc>,
	updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)]
pub enum PoolGameType {
	EightBall,
	NineBall,
	TenBall,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PoolGameWinner {
	Player1,
	Player2,
	Undetermined,
}

impl DBRecord for PoolGame {
	fn table() -> &'static str {
		"pool_games"
	}

	fn uuid(&self) -> UUID<Self> {
		self.uuid.to_owned()
	}
}

impl PoolGame {
	pub fn new(player1: UUID<PoolPlayer>, player2: UUID<PoolPlayer>, host: UUID<User>) -> Self {
		Self {
			uuid: UUID::default(),
			date: Utc::now().date_naive(),
			player1,
			player2,
			winner: PoolGameWinner::Undetermined,
			game_type: PoolGameType::EightBall,
			host,
			created_at: Utc::now(),
			updated_at: Utc::now(),
		}
	}
}
