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
        if value >= 1 && value <= 4 {
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
        if value >= 1 && value <= 2 {
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

#[derive(Debug)]
struct Parameters {
    effect: (Effect, u8),
    speed: u8,
    brightness: u8,
    wave_direction: (u8, u8),
}

fn parse_parameters(args: &mut pico_args::Arguments) -> Result<Parameters, String> {
    // parse effect into an enum
    let effect = match args.subcommand() {
        Ok(Some(s)) => effect_from_str(&s),
        Ok(None) => Err(format!("missing effect command")),
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

    Ok(Parameters {
        effect,
        speed,
        brightness,
        wave_direction,
    })
}

fn build_control_buffer(
    effect: (Effect, u8),
    speed: u8,
    brightness: u8,
    wave_direction: (u8, u8),
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
        Effect::Static | Effect::Breath => {
            // TODO: parse colors
            buf.append(&mut vec![0xFF; 12]);
        }
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

Supported modes:
    off                             Turn all keyboard backlight off.
    static                          Show static colored light for each zone.
    breath                          Fade light in and out.
    wave                            Directed left or right rainbow animation.
    hue                             Continuously cycle between hues.

USAGE:
    l5p-kbl-rs mode [options]

OPTIONS:
    common to all modes
        -b | --brightness <[1,2]>   Brightness: 1 = dimmer, 2 = brighter

    modes 'breath` | 'wave' | 'hue'
        -s | --speed <[1..4]>       Animation frequency: 1 = slowest, 4 = fastest

    mode 'wave'
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
    if let Some(device_handle) = rusb::open_device_with_vid_pid(VENDOR_ID, PRODUCT_ID) {
        match parse_parameters(&mut args) {
            Ok(p) => {
                println!("parameters: {:#?}", &p);
                let buf = build_control_buffer(p.effect, p.speed, p.brightness, p.wave_direction);
                match device_handle.write_control(
                    REQUEST_TYPE,
                    REQUEST,
                    VALUE,
                    INDEX,
                    &buf,
                    std::time::Duration::from_secs(1),
                ) {
                    Ok(_) => {}
                    Err(e) => println!("operation unsuccessful: {}", e),
                }
            }
            Err(msg) => println!("unable to parse parameters: {}", msg),
        }
    } else {
        println!("error: lighting device not found");
    }
}
