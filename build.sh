#!/bin/bash

# build library and put into godot project
(cd rusty_game_jam_lib ; cargo build)
rm rusty_game_jam_godot/librusty_game_jam_lib.so
cp rusty_game_jam_lib/target/debug/librusty_game_jam_lib.so rusty_game_jam_godot/