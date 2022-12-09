# kava.elg.gg (WIP)
A [Rocket](https://github.com/SergioBenitez/Rocket/) website and [Serenity](https://github.com/serenity-rs/serenity) Discord bot for a kava bar community.

## Setup
### Prerequisites
This project requires [Rust](https://www.rust-lang.org/tools/install) and [MySQL](https://dev.mysql.com/doc/mysql-installation-excerpt/8.0/en/).

### Clone, rename, and build
Run:
```
$ cd /
$ git clone git@github.com:Elgenzay/kava.elg.gg.git
$ mv /kava.elg.gg /kava
$ /kava/sh/build-release.sh
```

### Create symlinks
```
$ cd /kava/tls
$ ln -s /path/to/your/fullchain ./fullchain.pem
$ ln -s /path/to/your/privkey.pem ./privkey.pem
```

### Create Bot Config file
Create `/kava/bot/BotConfig.json`:

A reference file is available at `/kava/bot/BotConfig_sample.json`

### Create the MySQL database & user
Run in `$ mysql`:

```
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
```
CREATE USER 'mysql'@'%' IDENTIFIED BY '{MYSQL_PASS}';
GRANT ALL PRIVILEGES ON kava.* TO 'mysql'@'%';
```

### Set environment variables
Create `/kava/.env`:
```
BOT_TOKEN = "{TOKEN}"
WEBHOOK_URL = "https://discord.com/api/webhooks/{ETC}"
ICON_URL = "https://kava.elg.gg/icon.png"
DOMAIN = "kava.elg.gg"
MYSQL_PASS = "{PASSWORD}"
```

### Create run scripts (for development server)

Create `/kava/sh/run-redirect.sh`:
```
cd /kava/https-redirect
ROCKET_ADDRESS={IP_HERE} cargo run
```
Create `/kava/sh/run-web.sh`:
```
cd /kava/web
ROCKET_ADDRESS={IP_HERE} cargo run
```

### Create systemd services (for production server)
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
Create `/etc/systemd/system/kava_redirect.service`:
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


Run:
```
$ systemctl enable kava_web
$ systemctl enable kava_redirect
$ systemctl enable kava_bot

$ systemctl start kava_web
$ systemctl start kava_redirect
$ systemctl start kava_bot
```
