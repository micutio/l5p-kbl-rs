//! Rust version of https://github.com/imShara/l5p-kbl
//!
//! Lenovo Legion 5 Pro 2021 keyboard light controller
//! Shara, 2021, MIT
//!
//! Add udev rule as "/etc/udev/rules.d/10-kblight.rules" if you want control light as user
//! SUBSYSTEM=="usb", ATTR{idVendor}=="048d", ATTR{idProduct}=="c965", MODE="0666"
//!
//! Payload description
//!
//! HEADER             cc
//! HEADER             16
//! EFFECT             01 - static / 03 - breath / 04 - wave / 06 - hue
//! SPEED              01 / 02 / 03 / 04
//! BRIGHTNESS         01 / 02
//! RED SECTION 1      00-ff
//! GREEN SECTION 1    00-ff
//! BLUE SECTION 1     00-ff
//! RED SECTION 2      00-ff
//! GREEN SECTION 2    00-ff
//! BLUE SECTION 2     00-ff
//! RED SECTION 3      00-ff
//! GREEN SECTION 3    00-ff
//! BLUE SECTION 3     00-ff
//! RED SECTION 4      00-ff
//! GREEN SECTION 4    00-ff
//! BLUE SECTION 4     00-ff
//! UNUSED             00
//! WAVE MODE RTL      00 / 01
//! WAVE MODE LTR      00 / 01
//! UNUSED             00
//! UNUSED             00
//! UNUSED             00
//! UNUSED             00
//! UNUSED             00
//! UNUSED             00
//! UNUSED             00
//! UNUSED             00
//! UNUSED             00
//! UNUSED             00
//! UNUSED             00
//! UNUSED             00
//! UNUSED             00
//!

const VENDOR_ID: u16 = 0x048D;
const PRODUCT_ID: u16 = 0xC965;
const REQUEST_TYPE: u8 = 0x21;
const REQUEST: u8 = 0x9;
const VALUE: u16 = 0x03CC;
const INDEX: u16 = 0x00;

#[derive(Debug)]
enum Effect {
    Off,
    Static,
    Breath,
    Wave,
    Hue,
}

fn effect_from_str(effect: &str) -> Result<(Effect, u8), String> {
    match effect {
        "off" => Ok((Effect::Off, 1)),
        "static" => Ok((Effect::Static, 1)),
        "breath" => Ok((Effect::Breath, 3)),
        "wave" => Ok((Effect::Wave, 4)),
        "hue" => Ok((Effect::Hue, 6)),
        _ => Err(format!(
            "invalid effect: '{}'\nchoose either 'static', 'breath', 'wave' or 'hue'",
            effect,
        )),
    }
}

fn validate_speed_range(opt_val: Option<u8>) -> Result<u8, String> {
    if let Some(value) = opt_val {
        if (1..=4).contains(&value) {
            Ok(value)
        } else {
            Err(format!("speed must be in range [1..4], found {}", value))
        }
    } else {
        Ok(1)
    }
}

fn validate_brightness_range(opt_val: Option<u8>) -> Result<u8, String> {
    if let Some(value) = opt_val {
        if (1..=2).contains(&value) {
            Ok(value)
        } else {
            Err(format!("brightness must be either 1 or 2, found {}", value))
        }
    } else {
        Ok(1)
    }
}

fn validate_direction(opt_dir: Option<String>) -> Result<(u8, u8), String> {
    if let Some(dir) = opt_dir {
        match dir.as_str() {
            "ltr" => Ok((0, 1)),
            "rtl" => Ok((1, 0)),
            s => Err(format!(
                "direction must be either 'ltr' or 'rtl', found {}",
                s
            )),
        }
    } else {
        Ok((0, 0))
    }
}

fn parse_colors(args: Vec<std::ffi::OsString>) -> Result<Vec<(u8, u8, u8)>, String> {
    let colors = match args
        .into_iter()
        .map(from_hex)
        .collect::<Result<Vec<(u8, u8, u8)>, String>>()
    {
        Ok(vec) => vec,
        Err(s) => return Err(s),
    };

    if colors.is_empty() {
        Err("no colors found".to_string())
    } else {
        Ok(colors)
    }
}

fn consume(code: &mut std::str::Chars) -> Result<u32, String> {
    match code.next() {
        Some(x) => match x.to_digit(16) {
            Some(x) => Ok(x),
            None => Err("HtmlColorConversionError::InvalidCharacter".into()),
        },
        None => Err("HtmlColorConversionError::InvalidStringLength".into()),
    }
}

fn from_hex(code: std::ffi::OsString) -> Result<(u8, u8, u8), String> {
    // code_str.
    match code.into_string() {
        Ok(code_str) => {
            let mut chars = code_str.chars();

            let red1 = consume(&mut chars)?;
            let red2 = consume(&mut chars)?;
            let green1 = consume(&mut chars)?;
            let green2 = consume(&mut chars)?;
            let blue1 = consume(&mut chars)?;
            let blue2 = consume(&mut chars)?;

            Ok((
                (red1 * 16 + red2) as u8,
                (green1 * 16 + green2) as u8,
                (blue1 * 16 + blue2) as u8,
            ))
        }
        Err(_) => Err("OsString to String conversion error".to_string()),
    }
}

fn pad_colors(colors: &mut Vec<(u8, u8, u8)>) {
    if colors.len() < 4 {
        colors.reserve_exact(4 - colors.len());
    }
    while colors.len() < 4 {
        colors.extend_from_within(colors.len() - 1..colors.len())
    }
}

#[derive(Debug)]
struct Parameters {
    effect: (Effect, u8),
    speed: u8,
    brightness: u8,
    wave_direction: (u8, u8),
    colors: Vec<(u8, u8, u8)>,
}

fn parse_parameters(mut args: pico_args::Arguments) -> Result<Parameters, String> {
    // parse effect into an enum
    let effect = match args.subcommand() {
        Ok(Some(s)) => effect_from_str(&s),
        Ok(None) => Err("missing effect command".to_string()),
        Err(e) => Err(e.to_string()),
    }?;
    // parse speed and check whether it's in range
    let speed = match args.opt_value_from_str(["-s", "--speed"]) {
        Ok(val) => validate_speed_range(val),
        Err(e) => Err(e.to_string()),
    }?;
    // parse brightness and check whether it's in range
    let brightness = match args.opt_value_from_str(["-b", "--brightness"]) {
        Ok(val) => validate_brightness_range(val),
        Err(e) => Err(e.to_string()),
    }?;
    // parse wave direction
    let wave_direction = match args.opt_value_from_str(["-d", "--dir"]) {
        Ok(val) => validate_direction(val),
        Err(e) => Err(e.to_string()),
    }?;
    // parse colors
    let mut colors = parse_colors(args.finish())?;

    pad_colors(&mut colors);

    Ok(Parameters {
        effect,
        speed,
        brightness,
        wave_direction,
        colors,
    })
}

fn build_control_buffer(
    effect: (Effect, u8),
    speed: u8,
    brightness: u8,
    wave_direction: (u8, u8),
    colors: Vec<(u8, u8, u8)>,
) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::with_capacity(32);

    buf.push(204);
    buf.push(22);

    if let Effect::Off = effect.0 {
        buf.push(effect.1);
        buf.fill_with(|| 0);
        return buf;
    }

    buf.push(effect.1);
    buf.push(speed);
    buf.push(brightness);

    match effect.0 {
        Effect::Static | Effect::Breath => colors.iter().for_each(|(r, g, b)| {
            buf.push(*r);
            buf.push(*g);
            buf.push(*b);
        }),
        _ => {
            buf.append(&mut vec![0; 12]);
        }
    }

    // unused
    buf.push(0);

    buf.push(wave_direction.0);
    buf.push(wave_direction.1);

    // unused
    buf.append(&mut vec![0; 13]);

    buf
}

fn print_help() {
    println!(
        r#"
Lenovo Legion 5 Pro 2021 keyboard light controller
Inspired by https://github.com/imShara/l5p-kbl/
2022 Michael Wagner

Supported modes:
    off                             Turn all keyboard backlight off.
    static                          Show static coloured light for each zone.
    breath                          Fade light in and out.
    wave                            Directed left or right rainbow animation.
    hue                             Continuously cycle between hues.

USAGE:
    l5p-kbl-rs mode [options] colour1 [colour2] [colour3] [colour4]

    Colours are given as RGB tripels in hexadecimal form, e.g.: FF00ed,
    corresponding to each of the four regions on the keyboard, from left to
    right. If less than four colors are given, then the last colour is repeated
    for the remaining areas.

OPTIONS:
    common to all modes
        -b | --brightness <[1,2]>   Brightness: 1 = dimmer, 2 = brighter

    breath | wave | hue
        -s | --speed <[1..4]>       Animation frequency: 1 = slower, 4 = faster

    mode wave
        -d | --dir 'ltr' | 'rtl'    Set wave animation to go from left to right
                                    or right to left.
    "#
    );
}

fn main() {
    let mut args = pico_args::Arguments::from_env();

    if args.contains(["-h", "--help"]) {
        print_help();
        std::process::exit(0);
    }

    // println!("{:#?}", parse_parameters(args));

    let exit_code = match rusb::open_device_with_vid_pid(VENDOR_ID, PRODUCT_ID) {
        Some(device_handle) => match parse_parameters(args) {
            Ok(p) => {
                // TODO: Find minimal logging frameworks
                // println!("parameters: {:#?}", &p);
                let buf = build_control_buffer(
                    p.effect,
                    p.speed,
                    p.brightness,
                    p.wave_direction,
                    p.colors,
                );
                let timeout = std::time::Duration::from_secs(1);
                match device_handle.write_control(
                    REQUEST_TYPE,
                    REQUEST,
                    VALUE,
                    INDEX,
                    &buf,
                    timeout,
                ) {
                    Ok(_) => 0,
                    Err(e) => {
                        eprintln!("operation unsuccessful: {}", e);
                        1
                    }
                }
            }
            Err(msg) => {
                eprintln!("unable to parse parameters: {}", msg);
                1
            }
        },
        None => {
            eprintln!("error: lighting device not found");
            1
        }
    };
    std::process::exit(exit_code);
}
