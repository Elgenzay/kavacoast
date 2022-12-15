mod cmds;

use serde::{Deserialize, Serialize};
use serenity::async_trait;
use serenity::client::bridge::gateway::ShardManager;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{CommandResult, StandardFramework};
use serenity::model::channel::{Message, Reaction, ReactionType};
use serenity::model::gateway::Ready;
use serenity::model::guild::Member;
use serenity::model::prelude::command::Command;
use serenity::model::prelude::interaction::{Interaction, InteractionResponseType};
use serenity::model::prelude::{ChannelId, GuildChannel, GuildId, RoleId};
use serenity::prelude::Mutex;
use serenity::prelude::*;
use serenity::utils::ArgumentConvert;
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
				error_channel_id: 0,
				guild_id: 0,
			},
		}
	}
}

#[derive(Serialize, Deserialize, Clone)]
struct JsonData {
	react_role_groups: Vec<ReactRoleGroup>,
	error_channel_id: u64,
	guild_id: u64,
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

#[group]
#[commands(ping)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
	async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
		if let Interaction::ApplicationCommand(command) = interaction {
			let content = match command.data.name.as_str() {
				"ping" => cmds::ping::run(&command.data.options),
				_ => "".to_string(),
			};
			if let Err(e) = command
				.create_interaction_response(&ctx.http, |response| {
					response
						.kind(InteractionResponseType::ChannelMessageWithSource)
						.interaction_response_data(|message| message.content(content))
				})
				.await
			{
				log_error(
					&ctx,
					format!(
						"Error responding to command /{}: {}",
						command.data.name.as_str(),
						e
					),
				)
				.await;
			}
		}
	}

	async fn message(&self, ctx: Context, msg: Message) {
		get_state(&ctx).await;
		if msg.author.bot && msg.author.name == "KavaBot" && msg.content.contains("react") {
			for emoji in ["✅", "❎", "❤️"] {
				if let Err(e) = msg
					.react(&ctx.http, ReactionType::Unicode(String::from(emoji)))
					.await
				{
					log_error(&ctx, format!("Error reacting to message: {}", e)).await;
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

	async fn ready(&self, ctx: Context, ready: Ready) {
		let cmd_register = Command::create_global_application_command(&ctx.http, |command| {
			cmds::ping::register(command)
		})
		.await;
		if let Err(e) = cmd_register {
			log_error(&ctx, e.to_string()).await;
			panic!("Error registering commands: {}", e.to_string());
		};
		println!("{} is connected!", ready.user.name);
	}
}

async fn reaction_update(ctx: Context, react: Reaction, adding: bool) {
	let result =
		async {
			let groups = get_state(&ctx).await.react_role_groups;
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
		log_error(
			&ctx,
			format!("Error on reaction update: {}", result.err().unwrap()),
		)
		.await;
	}
}

async fn get_state(ctx: &Context) -> JsonData {
	let data = ctx.data.read().await;
	let config = data.get::<BotData>().unwrap();
	if config.initialized {
		return config.clone().data;
	}
	std::mem::drop(data);
	let mut data = ctx.data.write().await;
	let config = data.get_mut::<BotData>().unwrap();
	config.initialized = true;
	let json_str = fs::read_to_string("BotConfig.json").expect("Error reading BotConfig.json");
	config.data = serde_json::from_str(&json_str).expect("Error parsing BotConfig.json");
	config.clone().data
}

async fn log_error(ctx: &Context, error_message: String) {
	let state = get_state(ctx).await;
	let channel_result = GuildChannel::convert(
		ctx,
		Some(GuildId::from(state.guild_id)),
		Some(ChannelId::from(state.error_channel_id)),
		&state.error_channel_id.to_string()[..],
	)
	.await;
	let channel = match channel_result {
		Ok(v) => v,
		Err(_e) => {
			println!("Error channel not found. Error message: {}", error_message);
			return;
		}
	};
	match channel.say(&ctx.http, error_message).await {
		Ok(_) => (),
		Err(e) => println!("Error: {}", e.to_string()),
	};
}

#[tokio::main]
async fn main() {
	dotenv::dotenv().ok();
	let token = std::env::var("BOT_TOKEN").expect("Missing environment variable: BOT_TOKEN");
	let intents = GatewayIntents::non_privileged()
		| GatewayIntents::GUILD_MESSAGES
		| GatewayIntents::GUILD_MESSAGE_REACTIONS
		| GatewayIntents::MESSAGE_CONTENT;
	let framework = StandardFramework::new()
		.configure(|c| c.prefix("k!"))
		.group(&GENERAL_GROUP);
	let mut client = Client::builder(token, intents)
		.event_handler(Handler)
		.framework(framework)
		.type_map_insert::<BotData>(BotState::new())
		.await
		.expect("Error creating client");
	if let Err(e) = client.start().await {
		println!("Client error: {}", e.to_string());
	}
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
	msg.reply(ctx, "Pong!").await?;
	Ok(())
}
