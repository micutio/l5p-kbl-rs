//! Rust version of https://github.com/imShara/l5p-kbl
//!
//! Lenovo Legion 5 Pro 2021 keyboard light controller
//! Michael Wagner 2022
//!
//! Add udev rule as "/etc/udev/rules.d/10-kblight.rules" if you want control light as user
//! SUBSYSTEM=="usb", ATTR{idVendor}=="048d", ATTR{idProduct}=="c965", MODE="0666"
//!

#[cfg(feature = "gmonitor")]
mod g_monitor;
mod led;
mod msg;

fn cmd_set(mut args: pico_args::Arguments) -> i32 {
    if args.contains(["-h", "--help"]) {
        println!("{}", msg::HELP_SET);
        return 0;
    }

    match led::parse_parameters(args) {
        Ok(p) => led::set_led(p),
        Err(msg) => {
            eprintln!("error parsing params: {}", msg);
            1
        }
    }
}

#[cfg(not(feature = "gmonitor"))]
fn cmd_monitor(_args: pico_args::Arguments) -> i32 {
    1
}

#[cfg(feature = "gmonitor")]
fn cmd_monitor(mut args: pico_args::Arguments) -> i32 {
    if args.contains(["-h", "--help"]) {
        println!("{}", msg::HELP_MONITOR);
        return 0;
    }

    // parse key
    let path: String = match args.opt_value_from_str(["-f", "--file"]) {
        Ok(opt_str) => match opt_str {
            Some(s) => s,
            None => {
                eprintln!("no file path provided");
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("{}", e.to_string());
            std::process::exit(1);
        }
    };

    let monitors = match g_monitor::parse_from_file(path) {
        Ok(monitors) => monitors,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    for mut m in monitors {
        m.wait();
        m.close().kill().unwrap();
    }
    0
}

fn main() {
    let mut args = pico_args::Arguments::from_env();

    let exit_code = match args.subcommand() {
        Ok(Some(s)) => {
            if s.eq("set") {
                cmd_set(args)
            } else if s.eq("monitor") {
                cmd_monitor(args)
            } else {
                eprintln!("unknown command: {}", s);
                1
            }
        }
        Ok(None) => {
            if args.contains(["-h", "--help"]) {
                println!("{}", msg::HELP_GENERAL);
                std::process::exit(0);
            } else {
                eprintln!("missing command");
                1
            }
        }
        Err(e) => {
            eprintln!("error: {}", e);
            1
        }
    };

    std::process::exit(exit_code);
}
