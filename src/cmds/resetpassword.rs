use crate::dbrecord::DBRecord;
use serenity::{
	all::User,
	builder::{CreateCommand, CreateMessage},
	client::Context,
};

const RANDOM_PASSWORD_LENGTH: usize = 32;

pub async fn run(ctx: &Context, discord_user: &User) -> String {
	let user_id = discord_user.id.get().to_string();

	match crate::models::user::User::db_search_one("discord_id", user_id.clone()).await {
		Ok(user) => {
			if let Some(mut user) = user {
				let new_password =
					crate::generic::random_alphanumeric_string(RANDOM_PASSWORD_LENGTH);

				match user.set_password(&new_password).await {
					Ok(_) => {
						let msg = format!(
							"Your password has been reset. Here are your new credentials:\n\nUsername: `{}`\nPassword:\n||```\n{}\n```||\n\nhttps://kavacoast.com/login",
							user.username, new_password
						);

						let builder = CreateMessage::new().content(msg);

						match discord_user.direct_message(ctx, builder).await {
							Ok(_) => "Sent you a DM!".to_owned(),
							Err(_) => {
								"I can't DM you: https://support.discord.com/hc/en-us/articles/360060145013"
									.to_owned()
							}
						}
					}
					Err(e) => {
						log::error!("Password reset error updating password: {}", e);
						"Internal server error".to_owned()
					}
				}
			} else {
				"You're not registered. Use `/register` to get started.".to_owned()
			}
		}
		Err(e) => {
			log::error!("Password reset error on user search: {}", e);
			"Internal server error".to_owned()
		}
	}
}

pub fn register() -> CreateCommand {
	CreateCommand::new("resetpassword").description("Reset your password")
}
