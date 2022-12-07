#! /bin/bash
parent_path=$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )

cd "$parent_path"
cd ../web
cargo build --release
cd ../https-redirect
cargo build --release
cd ../discord-webhook
cargo build --release
cd ../bot
cargo build --release
cd "$parent_path"
echo "Release builds complete."
