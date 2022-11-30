#! /bin/bash
parent_path=$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )

cd "$parent_path"
cd ./web
cargo build
cd ../discord-webhook
cargo build
cd ../bot
cargo build
cd "$parent_path"
echo "Builds complete."
