use crate::cmds;
use crate::generic::Environment;
use chrono::{Datelike, TimeZone, Utc, Weekday};
use serde::{Deserialize, Serialize};
use serenity::all::Interaction;
use serenity::async_trait;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::framework::standard::StandardFramework;
use serenity::gateway::ShardManager;
use serenity::model::channel::{Reaction, ReactionType};
use serenity::model::gateway::Ready;
use serenity::model::guild::Member;
use serenity::model::prelude::RoleId;
use serenity::prelude::Mutex;
use serenity::prelude::*;
use serenity::utils::ArgumentConvert;
use std::fs;
use std::sync::Arc;
use std::time::Duration;
use tokio::{task, time};

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
			data: JsonData::new(),
		}
	}
}

#[derive(Serialize, Deserialize, Clone)]
struct JsonData {
	react_role_groups: Vec<ReactRoleGroup>,
	tickrate_seconds: u64,
	offset_hours: i64,
}

impl JsonData {
	fn new() -> JsonData {
		JsonData {
			react_role_groups: vec![],
			tickrate_seconds: 0,
			offset_hours: 0,
		}
	}
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
	async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
		if let Interaction::Command(command) = interaction {
			let content = match command.data.name.as_str() {
				"ping" => Some(cmds::ping::run(command.data.options().as_slice())),
				"register" => Some(cmds::register::run(&ctx, &command.user).await),
				"resetpassword" => Some(cmds::resetpassword::run(&ctx, &command.user).await),
				_ => Some(String::new()),
			};

			if let Some(content) = content {
				let data = CreateInteractionResponseMessage::new().content(content);
				let builder = CreateInteractionResponse::Message(data);

				if let Err(e) = command.create_response(&ctx.http, builder).await {
					log::error!(
						"Error responding to command /{}: {}",
						command.data.name.as_str(),
						e
					);
				}
			}
		}
	}

	//async fn message(&self, ctx: Context, msg: Message) {}

	async fn reaction_add(&self, ctx: Context, react: Reaction) {
		reaction_update(ctx, react, true).await;
	}

	async fn reaction_remove(&self, ctx: Context, react: Reaction) {
		reaction_update(ctx, react, false).await;
	}

	async fn ready(&self, ctx: Context, ready: Ready) {
		let state = get_state(&ctx).await;

		let commands = vec![
			cmds::register::register,
			cmds::ping::register,
			cmds::resetpassword::register,
		];

		for register_command in commands {
			let cmd_register =
				serenity::all::Command::create_global_command(&ctx.http, register_command()).await;
			if let Err(e) = cmd_register {
				panic!("Error registering command: {}", e);
			}
		}

		let task = async move {
			let mut interval =
				time::interval(Duration::from_millis(state.data.tickrate_seconds * 1000));
			loop {
				interval.tick().await;
				tick(&ctx).await;
			}
		};

		log::info!("{} is connected!", ready.user.name);

		if task::spawn(task).await.is_err() {
			panic!("Tokio task spawn failure");
		}
	}
}

async fn reaction_update(ctx: Context, react: Reaction, adding: bool) {
	let result =
		async {
			let groups = get_state(&ctx).await.data.react_role_groups;
			let msg_id = react.message_id.get();
			let mut match_group_opt = None;

			for group in groups {
				if group.message_id == msg_id {
					match_group_opt = Some(group);
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

			let mut role_id_opt = None;
			let mut remove_role_ids = vec![];

			for reactrole in match_group.roles {
				if reaction_str == &reactrole.emoji {
					role_id_opt = Some(reactrole.role_id);
				} else {
					remove_role_ids.push(reactrole.role_id);
				}
			}

			let role_id = match role_id_opt {
				Some(v) => v,
				None => return Ok(()),
			};

			let user_id = match &react.user_id {
				Some(v) => v,
				None => return Ok(()),
			};

			let user_id_str = &user_id.to_string();

			let member =
				match Member::convert(&ctx, react.guild_id, Some(react.channel_id), user_id_str)
					.await
				{
					Ok(v) => v,
					Err(e) => return Err(e.to_string()),
				};

			if match_group.mutually_exclusive {
				for remove_role in remove_role_ids {
					if let Err(e) = member
						.remove_role(&ctx.http, RoleId::new(remove_role))
						.await
					{
						return Err(e.to_string());
					};
				}
			}

			if adding {
				if let Err(e) = member.add_role(&ctx.http, RoleId::new(role_id)).await {
					return Err(e.to_string());
				};
			} else if let Err(e) = member.remove_role(&ctx.http, RoleId::new(role_id)).await {
				return Err(e.to_string());
			}

			Ok(())
		}
		.await;
	if let Err(e) = result {
		log::error!("Error on reaction update: {}", e);
	}
}

async fn get_state(ctx: &Context) -> BotState {
	let data = ctx.data.read().await;
	let state = match data.get::<BotData>() {
		Some(v) => v,
		None => {
			panic!("(Fatal) get_state(): State data not found");
		}
	};
	if state.initialized {
		return state.clone();
	}
	std::mem::drop(data);
	reset_state(ctx).await
}

async fn reset_state(ctx: &Context) -> BotState {
	let mut data = ctx.data.write().await;
	let state = match data.get_mut::<BotData>() {
		Some(v) => v,
		None => {
			panic!("(Fatal) reset_state(): State data not found");
		}
	};
	state.initialized = true;
	let json_str = match fs::read_to_string("BotConfig.json") {
		Ok(v) => v,
		Err(_) => {
			panic!("(Fatal) Error reading BotConfig.json");
		}
	};
	state.data = match serde_json::from_str(&json_str) {
		Ok(v) => v,
		Err(_) => panic!("Error parsing BotConfig.json"),
	};
	state.clone()
}

pub async fn start_bot() {
	let token = Environment::new().bot_token.val();

	let intents = GatewayIntents::non_privileged()
		| GatewayIntents::GUILD_MESSAGES
		| GatewayIntents::GUILD_MESSAGE_REACTIONS
		| GatewayIntents::MESSAGE_CONTENT;

	let framework = StandardFramework::new();

	let mut client = Client::builder(token, intents)
		.event_handler(Handler)
		.framework(framework)
		.type_map_insert::<BotData>(BotState::new())
		.await
		.expect("Error creating client");

	if let Err(e) = client.start().await {
		log::error!("Client error: {}", e);
	}
}

fn _get_weekday(offset: &i64) -> Weekday {
	Utc.timestamp_opt(chrono::offset::Utc::now().timestamp() + (3600 * offset), 0)
		.unwrap()
		.date_naive()
		.weekday()
}

async fn tick(_ctx: &Context) {}
