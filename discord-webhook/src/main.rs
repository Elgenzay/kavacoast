use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let mut map = HashMap::new();
	map.insert("username", "KavaBot");
	map.insert("content", "test");

	dotenv::dotenv().ok();
	let webhook_url =
		std::env::var("WEBHOOK_URL").expect("Missing environment variable: WEBHOOK_URL");
	let icon_url = std::env::var("ICON_URL").expect("Missing environment variable: ICON_URL");
	map.insert("avatar_url", &icon_url[..]);

	let client = reqwest::blocking::Client::new();
	let resp = client.post(webhook_url).json(&map).send();
	println!("{:#?}", resp);
	Ok(())
}
