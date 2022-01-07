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

fn build_control_string(
    effect: &str,
    speed: &str,
    brightness: &str,
    wave_direction: &str,
) -> Result<Vec<u8>, String> {
    let mut buf: Vec<u8> = Vec::with_capacity(32);

    buf.push(204);
    buf.push(22);

    if effect.eq("off") {
        buf.push(effect_from_str("static")?);
        buf.fill_with(|| 0);
        return Ok(buf);
    }

    buf.push(effect_from_str(effect)?);
    buf.push(speed_from_str(speed)?);
    buf.push(brightness_from_str(brightness)?);

    if effect.ne("static") && effect.ne("breath") {
        buf.append(&mut vec![0; 12]);
    } else {
        // TODO: parse colors
        buf.append(&mut vec![0xFF; 12]);
    }

    // unused
    buf.push(0);

    match wave_direction {
        "rtl" => buf.append(&mut vec![1, 0]),
        "ltr" => buf.append(&mut vec![0, 1]),
        _ => buf.append(&mut vec![0, 0]),
    }

    // unused
    buf.append(&mut vec![0; 13]);

    Ok(buf)
}

fn effect_from_str(effect: &str) -> Result<u8, String> {
    match effect {
        "static" => Ok(1),
        "breath" => Ok(3),
        "wave" => Ok(4),
        "hue" => Ok(6),
        _ => Err(format!(
            "invalid effect: {}, choose either 'static', 'breath', 'wave' or 'hue'",
            effect,
        )),
    }
}

fn speed_from_str(speed: &str) -> Result<u8, String> {
    match speed {
        "1" => Ok(1),
        "2" => Ok(2),
        "3" => Ok(3),
        "4" => Ok(4),
        _ => Err(format!(
            "invalid speed: {}, choose a number from 1 - 4",
            speed,
        )),
    }
}

fn brightness_from_str(brightness: &str) -> Result<u8, String> {
    match brightness {
        "1" => Ok(1),
        "2" => Ok(2),
        _ => Err(format!(
            "invalid brightness: {}, choose either 1 or 2",
            brightness,
        )),
    }
}

fn main() {
    if let Some(device_handle) = rusb::open_device_with_vid_pid(VENDOR_ID, PRODUCT_ID) {
        let device = device_handle.device();
        println!("found device at address {:04x}", device.address());

        match build_control_string("static", "1", "2", "rtl") {
            Ok(buf) => {
                match device_handle.write_control(
                    REQUEST_TYPE,
                    REQUEST,
                    VALUE,
                    INDEX,
                    &buf,
                    std::time::Duration::from_secs(1),
                ) {
                    Ok(n) => println!("{} bytes transferred", n),
                    Err(e) => println!("operation unsuccessful: {}", e),
                }
            }
            Err(msg) => println!("unable to create command: {}", msg),
        }
    } else {
        println!("error: device not found");
    }
}
