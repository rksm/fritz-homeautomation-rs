#!/usr/bin/env bash


# cargo run --bin fritz_dect_controller -- \
#       --user admin \
#       --password down8406 \
#       --ain "11657 0272633" \
#       --schedule data/schedule.txt

# ./target/release/fritz_dect_controller \
cargo run --bin fritz_dect_controller -- \
      --user admin \
      --password down8406 \
      --ain "11657 0272633" \
      --schedule data/schedule.txt
