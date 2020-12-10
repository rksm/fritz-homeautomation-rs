#!/usr/bin/env bash

HOST=rpi

ssh $HOST "bash --login -c 'mkdir -p projects/fritz-homeautomation-rust/'"
rsync --exclude=target -avzP --delete ./ $HOST:projects/fritz-homeautomation-rust/

ssh rpi "bash --login -c 'cd projects/fritz-homeautomation-rust/; cargo build --release; systemctl restart homeautomation-server.service; journalctl -u homeautomation-server.service -f'"
# ssh $HOST "bash --login -c 'cd projects/fritz-homeautomation-rust/; cargo build --release'"
