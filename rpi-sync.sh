#!/usr/bin/env bash

rsync --exclude=target -avzP --delete ./ rpi:projects/fritz-homeautomation-rust/

ssh rpi "bash --login -c 'cd projects/fritz-homeautomation-rust/; cargo build --release; systemctl restart homeautomation-server.service; journalctl -u homeautomation-server.service -f'"
