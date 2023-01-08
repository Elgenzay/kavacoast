# kava.elg.gg
This is a [Rocket](https://github.com/SergioBenitez/Rocket/) website and [Serenity](https://github.com/serenity-rs/serenity) Discord bot for a kava bar community in South Florida.

The website and Discord invite is at https://kava.elg.gg

# Setup
NOTE: THIS GUIDE IS OUTDATED

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

Download the MySQL `web-community` version: https://dev.mysql.com/downloads/installer/.

I personally use the MySQL **server only** on Windows in favor of using [DBeaver](https://dbeaver.io/) for database management.  
I'm using the **Development Computer** config type and left ports closed. I did not create a Windows service.


## Clone and build
`cd` to your preferred project directory and run:
```sh
$ git clone git@github.com:Elgenzay/kava.elg.gg.git
$ ./kava.elg.gg/sh/build-release.sh
```

## Configure TLS (or don't)
### With TLS:
[Get your certificate](https://certbot.eff.org/) and run:
```sh
$ cd ./kava.elg.gg/tls
$ ln -s /etc/letsencrypt/live/{DOMAIN}/fullchain.pem ./fullchain.pem
$ ln -s /etc/letsencrypt/live/{DOMAIN}/privkey.pem ./privkey.pem
```
### Without TLS:
Change the contents of `kava.elg.gg/web/Rocket.toml` to:
```toml
[default]
port = 80
limits = { form = "64 kB", json = "1 MiB" }
```

## Create Bot Config file
Create `kava.elg.gg/bot/BotConfig.json`

A reference file is available at `kava.elg.gg/bot/BotConfig_sample.json`

## Create the MySQL database & user
Run in `$ mysql`:
```mysql
CREATE DATABASE `kava`;
CREATE TABLE kava.`bartenders` (
  `id` int unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(64) NOT NULL,
  `hash` varchar(100) NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `id_UNIQUE` (`id`),
  UNIQUE KEY `name_UNIQUE` (`name`)
) ENGINE=InnoDB AUTO_INCREMENT=3 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
```
Password here should be the same as the `MYSQL_PASS` value in `/kava/.env` in the next step:
```mysql
CREATE USER 'kava'@'%' IDENTIFIED BY '{MYSQL_PASS}';
GRANT ALL PRIVILEGES ON kava.* TO 'kava'@'%';
```

## Set environment variables
Create `kava.elg.gg/.env`:
```env
BOT_TOKEN = "{BOT_TOKEN}"
WEBHOOK_URL = "{WEBHOOK_URL}"
ICON_URL = "https://kava.elg.gg/icon.png"
DOMAIN = "{DOMAIN}"
MYSQL_PASS = "{MYSQL_PASS}"
```
`DOMAIN` is not necessary when running locally.

You can get a `BOT_TOKEN` from the [Discord Developer Portal](https://discord.com/developers/), where you can also add the application to your Discord server.

You can generate a `WEBHOOK_URL` in the Discord app within a text channel's settings under **Integrations**.  
It looks like `https://discord.com/api/webhooks/...`

## Create run scripts (for development server)
Create `/kava/sh/run-web.sh`:
```sh
cd /kava/web
ROCKET_ADDRESS={IP_HERE} cargo run
```
Create `/kava/sh/run-redirect.sh` (unless running locally):
```sh
cd /kava/https-redirect
ROCKET_ADDRESS={IP_HERE} cargo run
```

## Create systemd services (for production server)
Create `/etc/systemd/system/kava_web.service`:
```
[Unit]
Description=Kava Web
After=mysqld.service
StartLimitBurst=5
StartLimitIntervalSec=0

[Service]
User=root
WorkingDirectory=/kava/web
Environment="ROCKET_ENV=prod"
Environment="ROCKET_ADDRESS={IP_HERE}"
Environment="ROCKET_LOG=critical"
ExecStart=/kava/web/target/release/web
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
WorkingDirectory=/kava/https-redirect
Environment="ROCKET_ENV=prod"
Environment="ROCKET_ADDRESS={IP_HERE}"
Environment="ROCKET_LOG=critical"
ExecStart=/kava/https-redirect/target/release/https-redirect
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
WorkingDirectory=/kava/bot
ExecStart=/kava/bot/target/release/kavabot
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

Note: The project is still under active development.
