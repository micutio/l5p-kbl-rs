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

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
enum Effect {
    Off,
    Static,
    Breath,
    Wave,
    Hue,
}

impl Effect {
    /// Map each effect to the corresponding byte.
    fn as_byte(effect: Effect) -> u8 {
        match effect {
            Effect::Off => 1,
            Effect::Static => 1,
            Effect::Breath => 3,
            Effect::Wave => 4,
            Effect::Hue => 6,
        }
    }
}

fn effect_from_str(effect: &str) -> Result<Effect, String> {
    match effect {
        "off" => Ok(Effect::Off),
        "static" => Ok(Effect::Static),
        "breath" => Ok(Effect::Breath),
        "wave" => Ok(Effect::Wave),
        "hue" => Ok(Effect::Hue),
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
        eprintln!("no colors found");
        Ok(vec![(0, 0, 0); 4])
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

/// Available parameters for configuring the keyboard LEDs.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Parameters {
    effect: Effect,
    speed: u8,
    brightness: u8,
    wave_direction: (u8, u8),
    colors: Vec<(u8, u8, u8)>,
}

/// Parse the LED parameter set from command line arguments.
pub fn parse_parameters(mut args: pico_args::Arguments) -> Result<Parameters, String> {
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

fn build_control_buffer(params: Parameters) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::with_capacity(32);

    buf.push(204);
    buf.push(22);

    if let Effect::Off = params.effect {
        buf.push(Effect::as_byte(params.effect));
        buf.fill_with(|| 0);
        return buf;
    }

    buf.push(Effect::as_byte(params.effect));
    buf.push(params.speed);
    buf.push(params.brightness);

    match params.effect {
        Effect::Static | Effect::Breath => params.colors.iter().for_each(|(r, g, b)| {
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

    buf.push(params.wave_direction.0);
    buf.push(params.wave_direction.1);

    // unused
    buf.append(&mut vec![0; 13]);

    buf
}

/// Set the keyboard LEDs to the given parameters.
/// Fail if the device cannot be
/// - found
/// - acquired with a handle
/// - written to
pub fn set_led(parameters: Parameters) -> i32 {
    if let Some(mut device_handle) = rusb::open_device_with_vid_pid(VENDOR_ID, PRODUCT_ID) {
        let buf = build_control_buffer(parameters);
        let timeout = std::time::Duration::from_secs(1);

        // in case of an active kernel driver we have to detach it first, lest we'll get an IO
        // error trying to send commands to the device
        match device_handle.kernel_driver_active(0) {
            Ok(is_active) => {
                if is_active {
                    device_handle.detach_kernel_driver(0).unwrap();
                }
            }
            Err(e) => {
                eprintln!("{}", e.to_string());
                return 1; // don't try anything further if we can't determine kernel driver status
            }
        }

        match device_handle.write_control(REQUEST_TYPE, REQUEST, VALUE, INDEX, &buf, timeout) {
            Ok(_) => 0,
            Err(e) => {
                eprintln!("{}", e.to_string());
                1
            }
        }
    } else {
        eprintln!("error: lighting device not found");
        1
    }
}
