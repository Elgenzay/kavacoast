# kava.elg.gg
This is a [Rocket](https://github.com/SergioBenitez/Rocket/) website and [Serenity](https://github.com/serenity-rs/serenity) Discord bot for a kava bar community in South Florida.

This project includes:
- A landing page
- A dashboard that allows bartenders to log in and update their schedules
- A Discord bot that announces which bartenders are working which shift at which location every day  
(if any bartenders are working that day) and automatically assigns roles for reactions on specified messages

The website and Discord invite is at https://kava.elg.gg

# Setup

If you want to try/implement/modify/contribute to/study it, here's a guide for setting up the environment.

In this guide, screaming snake case within curly brackets {LIKE_THIS} denote environment-specific values.
## Prerequisites
This project requires [Rust](https://www.rust-lang.org/tools/install) and [MySQL](https://dev.mysql.com/doc/mysql-installation-excerpt/8.0/en/) to be installed on the host machine.

On Unix, run:
```sh
$ curl https://sh.rustup.rs -sSf | sh
$ apt install mysql-server
```
On Windows:

Download Rust: https://www.rust-lang.org/learn/get-started

Download the MySQL `web-community` version: https://dev.mysql.com/downloads/installer/

I personally use the MySQL **server only** on Windows in favor of using [DBeaver](https://dbeaver.io/) for database management.  
I'm using the **Development Computer** config type and left ports closed.

The remainder of this guide can be followed without TLS with [Git Bash](https://git-scm.com/downloads) when setting up a local development environment on Windows.


## Clone and build
`cd` to your preferred project directory and run:
```sh
$ git clone git@github.com:Elgenzay/kava.elg.gg.git
$ cd kava.elg.gg/
$ cargo build --release
```

## Configure TLS (or don't)
### With TLS:
[Get your certificate](https://certbot.eff.org/) and run:
```sh
$ cd tls/
$ ln -s /etc/letsencrypt/live/{DOMAIN}/fullchain.pem fullchain.pem
$ ln -s /etc/letsencrypt/live/{DOMAIN}/privkey.pem privkey.pem
```
### Without TLS:
Change the contents of `kava.elg.gg/web/Rocket.toml` to:
```toml
[default]
port = 80
limits = { form = "64 kB", json = "1 MiB" }
```

## Create config files
Create files:  
`kava.elg.gg/bot/BotConfig.json` and  
`kava.elg.gg/web/static/resources/json/PublicData.json`

Reference files are available at:  
`kava.elg.gg/bot/BotConfig_sample.json` and  
`kava.elg.gg/web/static/resources/json/PublicData_sample.json`, respectively.

## Create the MySQL database & user
Run in `$ mysql`:
```mysql
CREATE DATABASE `kava`;
USE `kava`;
```
Create tables:
```mysql
CREATE TABLE `bartenders` (
  `id` int NOT NULL AUTO_INCREMENT,
  `name` varchar(64) NOT NULL,
  `hash` varchar(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL DEFAULT '0',
  `discord_id` bigint unsigned NOT NULL DEFAULT '0',
  PRIMARY KEY (`id`),
  UNIQUE KEY `id_UNIQUE` (`id`),
  UNIQUE KEY `name_UNIQUE` (`name`)
) ENGINE=InnoDB AUTO_INCREMENT=5 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE `log_queue` (
  `id` int NOT NULL AUTO_INCREMENT,
  `guild_id` bigint unsigned NOT NULL,
  `ch_id` bigint unsigned NOT NULL,
  `msg` varchar(1024) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL,
  `reactions` json NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=73 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE `schedule` (
  `id` int NOT NULL AUTO_INCREMENT,
  `location` varchar(100) DEFAULT NULL,
  `sun1` json DEFAULT NULL,
  `mon1` json DEFAULT NULL,
  `tue1` json DEFAULT NULL,
  `wed1` json DEFAULT NULL,
  `thu1` json DEFAULT NULL,
  `fri1` json DEFAULT NULL,
  `sat1` json DEFAULT NULL,
  `sun2` json DEFAULT NULL,
  `mon2` json DEFAULT NULL,
  `tue2` json DEFAULT NULL,
  `wed2` json DEFAULT NULL,
  `thu2` json DEFAULT NULL,
  `fri2` json DEFAULT NULL,
  `sat2` json DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `schedule_un` (`location`)
) ENGINE=InnoDB AUTO_INCREMENT=4 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
```

Password here should be the same as the `MYSQL_PASS` value in `/kava/.env` in the next step:
```mysql
CREATE USER 'kava'@'%' IDENTIFIED BY '{MYSQL_PASS}';
GRANT ALL PRIVILEGES ON kava.* TO 'kava'@'%';
```

When setting up, you will need to populate the `id` and `location` fields for each location you want to add.  
The `location` fields should match the `name` value of the corresponding shift within `kava.elg.gg/web/static/resources/json/PublicData.json`.  
This is an identifier string, rather than a friendly name. It won't be visible to end users.  
After setting up, run the `k!weekly` bot command twice to initialize the remaining fields.

To add bartenders, populate the `id` and `name` fields for each user, and set `hash` and `discord_id` to `0`.  
When `hash` is `0`, any password is valid when logging in.  
After the user is logged in, they can change their password on the dashboard.

## Set environment variables
Create `kava.elg.gg/.env`:
```env
BOT_TOKEN = "{BOT_TOKEN}"
WEBHOOK_URL = "{WEBHOOK_URL}"
DOMAIN = "{DOMAIN}"
MYSQL_PASS = "{MYSQL_PASS}"
DISCORD_INVITE_LINK = "{DISCORD_INVITE_LINK}"
DISCORD_LOG_CHANNEL_ID = "{DISCORD_LOG_CHANNEL_ID}"
DISCORD_ERROR_CHANNEL_ID = "{DISCORD_ERROR_CHANNEL_ID}"
DISCORD_SCHEDULE_CHANNEL_ID = "{DISCORD_SCHEDULE_CHANNEL_ID}"
DISCORD_GUILD_ID = "{DISCORD_GUILD_ID}"
ADMIN_ID = "{ADMIN_ID}"
```
`DOMAIN` is not necessary when running locally.

You can get a `BOT_TOKEN` from the [Discord Developer Portal](https://discord.com/developers/), where you can also add the application to your Discord server.

You can generate a `WEBHOOK_URL` in the Discord app within a text channel's settings under **Integrations**.  
It looks like `https://discord.com/api/webhooks/...`

`DISCORD_INVITE_LINK` is where the /join page redirects to, and looks like `https://discord.gg/...`. Make sure it's set to **not** expire.

Channel/guild IDs can be copied from the context menu by right-clicking channels/guilds with developer mode enabled:  
**User Settings > App Settings > Advanced > Developer mode**

If you're testing locally, you're done setting up.  
If you're setting up on a server and want it to stay running, continue reading.

## Create systemd services

This section assumes a project directory at `/kava.elg.gg/`.

Create `/etc/systemd/system/kava_web.service`:
```
[Unit]
Description=Kava Web
After=mysqld.service
StartLimitBurst=5
StartLimitIntervalSec=0

[Service]
User=root
WorkingDirectory=/kava.elg.gg/web
Environment="ROCKET_ENV=prod"
Environment="ROCKET_ADDRESS={IP_HERE}"
Environment="ROCKET_LOG=critical"
ExecStart=/kava.elg.gg/target/release/web
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```
Create `/etc/systemd/system/kava_redirect.service` (unless running without TLS):
```
[Unit]
Description=Kava Redirect
After=mysqld.service
StartLimitBurst=5
StartLimitIntervalSec=0

[Service]
User=root
WorkingDirectory=/kava.elg.gg/https-redirect
Environment="ROCKET_ENV=prod"
Environment="ROCKET_ADDRESS={IP_HERE}"
Environment="ROCKET_LOG=critical"
ExecStart=/kava.elg.gg/target/release/https-redirect
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```
Create `/etc/systemd/system/kava_bot.service`:
```
[Unit]
Description=KavaBot
StartLimitBurst=5
StartLimitIntervalSec=0

[Service]
User=root
WorkingDirectory=/kava.elg.gg/bot
ExecStart=/kava.elg.gg/target/release/bot
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

Start the services:
```sh
$ systemctl enable kava_web
$ systemctl enable kava_bot
$ systemctl start kava_web
$ systemctl start kava_bot
```

If running with TLS:
```sh
$ systemctl enable kava_redirect
$ systemctl start kava_redirect
```
