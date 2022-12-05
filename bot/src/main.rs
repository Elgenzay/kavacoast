use serenity::async_trait;
use serenity::model::channel::{Message, Reaction, ReactionType};
use serenity::model::gateway::Ready;
use serenity::model::guild::Member;
use serenity::model::prelude::Role;
use serenity::model::prelude::RoleId;
use serenity::prelude::*;
use serenity::utils::ArgumentConvert;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
	async fn message(&self, ctx: Context, msg: Message) {
		if msg.content == "!ping" {
			if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
				println!("Error sending message: {:?}", why);
			}
		}
		if msg.author.bot && msg.author.name == "KavaBot" && msg.content.contains("react") {
			for emoji in ["✅", "❎", "❤️"] {
				if let Err(why) = msg
					.react(&ctx.http, ReactionType::Unicode(String::from(emoji)))
					.await
				{
					println!("Error reacting to message: {:?}", why)
				};
			}
		}
	}

	async fn reaction_add(&self, ctx: Context, react: Reaction) {
		let result = async {
			let reaction_str = match &react.emoji {
				ReactionType::Unicode(s) => s,
				_ => return Ok(()),
			};
			let user_id = match &react.user_id {
				Some(v) => v,
				None => return Ok(()),
			};
			let user_id_str = &user_id.to_string()[..];
			let mut member =
				match Member::convert(&ctx, react.guild_id, Some(react.channel_id), user_id_str)
					.await
				{
					Ok(v) => v,
					Err(e) => return Err(e.to_string()),
				};
			if let Err(e) = member
				.add_role(&ctx.http, RoleId(1049310608622899261))
				.await
			{
				return Err(e.to_string());
			};

			Ok(())
		}
		.await;
		if result.is_err() {
			println!("Error: {}", result.err().unwrap());
		}
	}

	async fn ready(&self, _: Context, ready: Ready) {
		println!("{} is connected!", ready.user.name);
	}
}

#[tokio::main]
async fn main() {
	dotenv::dotenv().ok();
	let token = std::env::var("BOT_TOKEN").expect("Missing environment variable: BOT_TOKEN");
	let intents = GatewayIntents::GUILD_MESSAGES
		| GatewayIntents::GUILD_MESSAGE_REACTIONS
		| GatewayIntents::MESSAGE_CONTENT;
	let mut client = Client::builder(&token, intents)
		.event_handler(Handler)
		.await
		.expect("Err creating client");
	if let Err(why) = client.start().await {
		println!("Client error: {:?}", why);
	}
}
