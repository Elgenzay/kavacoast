use serenity::async_trait;
use serenity::model::channel::{Message, Reaction, ReactionType};
use serenity::model::gateway::Ready;
use serenity::model::guild::Member;
use serenity::model::prelude::RoleId;
use serenity::prelude::*;
use serenity::utils::ArgumentConvert;

use serenity::client::bridge::gateway::ShardManager;

use serde::Deserialize;
use serde::Serialize;
use serenity::prelude::Mutex;
use std::fs;
use std::sync::Arc;

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
	type Value = Arc<Mutex<ShardManager>>;
}

struct BotData;

impl TypeMapKey for BotData {
	type Value = BotState;
}

#[derive(Clone)]
struct BotState {
	initialized: bool,
	data: JsonData,
}

impl BotState {
	fn new() -> BotState {
		BotState {
			initialized: false,
			data: JsonData {
				react_role_groups: Vec::new(),
			},
		}
	}
}

#[derive(Serialize, Deserialize, Clone)]
struct JsonData {
	react_role_groups: Vec<ReactRoleGroup>,
}

#[derive(Serialize, Deserialize, Clone)]
struct ReactRoleGroup {
	message_id: u64,
	mutually_exclusive: bool,
	roles: Vec<ReactRole>,
}

#[derive(Serialize, Deserialize, Clone)]
struct ReactRole {
	emoji: String,
	role_id: u64,
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
	async fn message(&self, ctx: Context, msg: Message) {
		get_state(&ctx).await;
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
		reaction_update(ctx, react, true).await;
	}

	async fn reaction_remove(&self, ctx: Context, react: Reaction) {
		reaction_update(ctx, react, false).await;
	}

	async fn ready(&self, _ctx: Context, ready: Ready) {
		println!("{} is connected!", ready.user.name);
	}
}

async fn reaction_update(ctx: Context, react: Reaction, adding: bool) {
	let result =
		async {
			let groups = get_state(&ctx).await.data.react_role_groups;
			let msg_id = react.message_id.as_u64();
			let mut match_group_opt = None;
			for group in groups {
				if &group.message_id == msg_id {
					match_group_opt = Some(group.clone());
					break;
				}
			}
			let match_group = match match_group_opt {
				Some(v) => v,
				None => return Ok(()),
			};
			let reaction_str = match &react.emoji {
				ReactionType::Unicode(s) => s,
				_ => return Ok(()),
			};
			let mut role_id = None;
			let mut remove_role_ids = vec![];
			for reactrole in match_group.roles {
				if reaction_str == &reactrole.emoji {
					role_id = Some(reactrole.role_id);
				} else {
					remove_role_ids.push(reactrole.role_id);
				}
			}
			if role_id.is_none() {
				return Ok(());
			}
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
			if match_group.mutually_exclusive {
				for remove_role in remove_role_ids {
					if let Err(e) = member.remove_role(&ctx.http, RoleId(remove_role)).await {
						return Err(e.to_string());
					};
				}
			}
			if adding {
				if let Err(e) = member.add_role(&ctx.http, RoleId(role_id.unwrap())).await {
					return Err(e.to_string());
				};
			} else {
				if let Err(e) = member
					.remove_role(&ctx.http, RoleId(role_id.unwrap()))
					.await
				{
					return Err(e.to_string());
				};
			}
			Ok(())
		}
		.await;
	if result.is_err() {
		println!("Error: {}", result.err().unwrap());
	}
}

async fn get_state(ctx: &Context) -> BotState {
	let data = ctx.data.read().await;
	let config = data.get::<BotData>().unwrap();
	if config.initialized {
		return config.clone();
	}
	std::mem::drop(data);
	let mut data = ctx.data.write().await;
	let config = data.get_mut::<BotData>().unwrap();
	config.initialized = true;
	let json_str = fs::read_to_string("BotConfig.json").expect("Error reading BotConfig.json");
	config.data = serde_json::from_str(&json_str).expect("Error parsing BotConfig.json");
	config.clone()
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
		.type_map_insert::<BotData>(BotState::new())
		.await
		.expect("Err creating client");
	if let Err(why) = client.start().await {
		println!("Client error: {:?}", why);
	}
}
