use discord_log::Logger;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
	CommandDataOption, CommandDataOptionValue,
};

pub fn run(options: &[CommandDataOption]) -> String {
	let option = options
		.get(0)
		.expect("Expected function option")
		.resolved
		.as_ref()
		.expect("as_ref() result error");
	let opt_str = match option {
		CommandDataOptionValue::String(s) => s,
		_ => panic!(),
	};

	match &opt_str[..] {
		"invoke_log" => {
			let s = "Log test invoked.".to_owned();
			Logger::new().log_message(s.to_owned());
			s
		}
		"invoke_error" => {
			let s = "Error test invoked.".to_owned();
			Logger::new().log_error(s.to_owned());
			s
		}
		_ => "Invalid function identifier string".to_owned(),
	}
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	command
		.name("debug")
		.description("Invoke debug functions")
		.create_option(|option| {
			option
				.name("function")
				.description("The function identifier string")
				.kind(CommandOptionType::String)
				.required(true)
		})
}
