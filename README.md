# Lenovo Legion 5 Pro 2021 - Linux Keyboard Light RGB Controller

Rust version of the great Python script at https://github.com/imShara/l5p-kbl

## Installation

### Option 1 - Build from source

```sh
git clone https://github.com/Micutio/l5p-kbl-rs
```

By default the binary is built with the `set` function only.

```sh
cargo build --release
```

To include the gsettings monitor, build with the feature flag

```sh
cargo build --release --features gmonitor
```

## Usage

### `set`

```sh
l5p-kbl-rs set <LED mode> [options] colour1 [colour2] [colour3] [colour4]
```

Colours are given as RGB tripels in hexadecimal form, e.g.: FF00ed,
corresponding to each of the four regions on the keyboard, from left to
right. If less than four colors are given, then the last colour is repeated
for the remaining areas.

Supported LED modes:

- off:    Turn all keyboard backlight off.
- static: Show static coloured light for each zone.
- breath: Fade light in and out.
- wave:   Directed left or right rainbow animation.
- hue:    Continuously cycle between hues.

Options

- `-b | --brightness <[1,2]>`: Brightness: 1 = dimmer, 2 = brighter. Available to all modes.

- `-s | --speed <[1..4]>`:    Animation frequency: 1 = slower, 4 = faster. Does not apply to mode `static`

- `-d | --dir 'ltr' | 'rtl'`: Set wave animation to go from left to right or right to left. Only applies to mode `wave`
