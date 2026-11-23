#!/bin/sh
printf "%s\n" "Build Kineti 1.0.0 from source with 'cargo build --release --locked --manifest-path core-native/Cargo.toml', or install kineti-cli 1.0.0 via Cargo / kineti 1.0.0 via npm after approved registry publication. Then preview: kineti init --goal <explicit-goal>."
exit 2
