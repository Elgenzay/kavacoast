# kava.elg.gg (WIP)
A [Rocket](https://github.com/SergioBenitez/Rocket/) website and [Serenity](https://github.com/serenity-rs/serenity) Discord bot for a kava bar community.

## Setup
### Clone to root & rename to `kava`
```
cd /
git clone git@github.com:Elgenzay/kava.elg.gg.git
mv /kava.elg.gg /kava
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
cd /kava/tls
ln -s /path/to/your/fullchain ./fullchain.pem
ln -s /path/to/your/privkey.pem ./privkey.pem
```

### Create run scripts

`/kava/sh/run-redirect-release.sh`
```
cd /kava/https-redirect
ROCKET_ADDRESS={IP_HERE} cargo run --release
```
`/kava/sh/run-redirect.sh`
```
cd /kava/https-redirect
ROCKET_ADDRESS={IP_HERE} cargo run
```
`/kava/sh/run-web-release.sh`
```
cd /kava/web
ROCKET_ADDRESS={IP_HERE} cargo run --release
```
`/kava/sh/run-web.sh`
```
cd /kava/web
ROCKET_ADDRESS={IP_HERE} cargo run
```
