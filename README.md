# falling sand sim

this is a fun little sand simulation that i built to run entirely in the windows console using my [rusty_console_game_engine](https://github.com/rip-super/https://github.com/rip-super/RustyConsoleGameEngine)!!

### usage

1. go to the releases tab and run the exe in conhost.exe.

2. or build from source:
```
git clone https://github.com/rip-super/sand-sim
cd sand-sim
cargo run --release
```

### notes

this only works on windows 10/11. make sure to use conhost.exe to run it or the rendering will be broken.

### controls

* left click: use current tool
* 1: place sand
* 2: delete
* c: clear all
* up/down arrows: change brush size