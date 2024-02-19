use crate::{dbrecord::DBRecord, models::registration::Registration};
use serenity::{
	all::User,
	builder::{CreateCommand, CreateMessage},
	client::Context,
};

pub async fn run(ctx: &Context, user: &User) -> String {
	let user_id = user.id.get().to_string();

	match crate::models::user::User::db_search_one("discord_id", &user_id).await {
		Ok(user) => {
			if user.is_some() {
				return "You're already registered. Use `/resetpassword` if you've lost your credentials.".to_owned();
			}
		}
		Err(e) => {
			log::error!("User search error on registration: {}", e);
			return "Internal server error".to_owned();
		}
	}

	match Registration::db_search_one("discord_id", &user_id).await {
		Ok(Some(existing)) => return existing.dm_string(),
		Err(e) => {
			log::error!("Registration search error on registration: {}", e);
			return "Internal server error".to_owned();
		}
		Ok(None) => (),
	};

	match Registration::new(Some(user_id.to_owned()))
		.db_create()
		.await
	{
		Ok(registration) => {
			let builder = CreateMessage::new().content(registration.dm_string());

			match user.direct_message(ctx, builder).await {
				Ok(_) => "Sent you a DM!".to_owned(),
				Err(_) => {
					"I can't DM you: https://support.discord.com/hc/en-us/articles/360060145013"
						.to_owned()
				}
			}
		}
		Err(e) => {
			log::error!("Registration create error on registration: {}", e);
			"Internal server error".to_owned()
		}
	}
}

pub fn register() -> CreateCommand {
	CreateCommand::new("register").description("Receive a registration link")
}
