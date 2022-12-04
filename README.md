# kava.elg.gg (WIP)
A [Rocket](https://github.com/SergioBenitez/Rocket/) website and [Serenity](https://github.com/serenity-rs/serenity) Discord bot for a kava bar community.

## Setup
### Clone to root & rename to `kava`
```
$ cd /
$ git clone git@github.com:Elgenzay/kava.elg.gg.git
$ mv /kava.elg.gg /kava
```

### Create `/kava/.env`
```
BOT_TOKEN = "{TOKEN}"
WEBHOOK_URL = "https://discord.com/api/webhooks/{ETC}"
ICON_URL = "https://kava.elg.gg/icon.png"
DOMAIN = "kava.elg.gg"
```

### Create symlinks
```
$ cd /kava/tls
$ ln -s /path/to/your/fullchain ./fullchain.pem
$ ln -s /path/to/your/privkey.pem ./privkey.pem
```

### Create run scripts (for development server)

`/kava/sh/run-redirect.sh`
```
cd /kava/https-redirect
ROCKET_ADDRESS={IP_HERE} cargo run
```
`/kava/sh/run-web.sh`
```
cd /kava/web
ROCKET_ADDRESS={IP_HERE} cargo run
```

### Create systemd services (for production server)
`/etc/systemd/system/kava_web.service`
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
`/etc/systemd/system/kava_redirect.service`
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
Enable them:
```
$ systemctl enable kava_web
$ systemctl enable kava_redirect

$ systemctl start kava_web
$ systemctl start kava_redirect
```
