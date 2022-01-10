//! Rust version of https://github.com/imShara/l5p-kbl
//!
//! Lenovo Legion 5 Pro 2021 keyboard light controller
//! Michael Wagner 2022
//!
//! Add udev rule as "/etc/udev/rules.d/10-kblight.rules" if you want control light as user
//! SUBSYSTEM=="usb", ATTR{idVendor}=="048d", ATTR{idProduct}=="c965", MODE="0666"
//!

mod led;
mod monitor;

fn print_help() {
    println!(
        r#"
Lenovo Legion 5 Pro 2021 keyboard light controller
Inspired by https://github.com/imShara/l5p-kbl/
2022 Michael Wagner

USAGE:
    l5p-kbl-rs [set | monitor] 

    ---------------------------------------------------------------------------

    set: directly set led mode and attributes

    set mode [options] colour1 [colour2] [colour3] [colour4]

    Colours are given as RGB tripels in hexadecimal form, e.g.: FF00ed,
    corresponding to each of the four regions on the keyboard, from left to
    right. If less than four colors are given, then the last colour is repeated
    for the remaining areas.

    Supported LED modes:
    off                             Turn all keyboard backlight off.
    static                          Show static coloured light for each zone.
    breath                          Fade light in and out.
    wave                            Directed left or right rainbow animation.
    hue                             Continuously cycle between hues.

    set OPTIONS:
        
        -b | --brightness <[1,2]>   Brightness: 1 = dimmer, 2 = brighter
                                    Available to all modes.
    
        -s | --speed <[1..4]>       Animation frequency: 1 = slower, 4 = faster
                                    Only applies to modes: breath | wave | hue
        
        -d | --dir 'ltr' | 'rtl'    Set wave animation to go from left to right
                                    or right to left. Only applies to mode wave


    monitor: assign keyboard LED configurations to changes in system variables
    
    monitor [options]
    
    monitor OPTIONS:
        -f | --file <filepath>      read variable to led config mapping from
                                    provided JSON file                

    [setting domain], [setting key], [setting value substring], [led parameters]
    "#
    );
}

fn main() {
    let mut args = pico_args::Arguments::from_env();

    if args.contains(["-h", "--help"]) {
        print_help();
        std::process::exit(0);
    }

    if args.contains(["-D", "--daemon"]) {
        todo!("listen to gsettings for changes and adjust keybord light accordingly");

        // println!("{:#?}", led::parse_parameters(args));

        let mut child = monitor::listen(
            "org.gnome.desktop.peripherals.touchpad",
            "send-events",
            |line| {
                let l = line.unwrap();
                if l.contains("enabled") {
                    println!("KEYBOARD LIGHT ON");
                }
                if l.contains("disabled") {
                    println!("KEYBOARD LIGHT OFF");
                }
            },
        )
        .unwrap();

        match child.wait() {
            Ok(it) => todo!(),
            Err(err) => return todo!(),
        };

        // alternative: wait for user input to terminate the program
        // let mut buffer = String::new();
        // let mut stdin = io::stdin(); // We get `Stdin` here.
        // stdin.read_line(&mut buffer)?;
        // Ok(())
    }

    let exit_code = match led::parse_parameters(args) {
        Ok(p) => match led::set_led(p) {
            Ok(_) => 0,
            Err(msg) => {
                eprintln!("error setting kbd: {}", msg);
                1
            }
        },
        Err(msg) => {
            eprintln!("error parsing params: {}", msg);
            1
        }
    };
    std::process::exit(exit_code);
}
