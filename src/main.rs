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
mod msg;

fn subcommand_set(mut args: pico_args::Arguments) -> i32 {
    if args.contains(["-h", "--help"]) {
        println!("{}", msg::HELP_SET);
        std::process::exit(0);
    }

    match led::parse_parameters(args) {
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
    }
}

fn subcommand_monitor(mut args: pico_args::Arguments) -> i32 {
    if args.contains(["-h", "--help"]) {
        println!("{}", msg::HELP_MONITOR);
        std::process::exit(0);
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

    let monitors = match monitor::parse_from_file(path) {
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

    // alternative: wait for user input to terminate the program
    // let mut buffer = String::new();
    // let mut stdin = io::stdin(); // We get `Stdin` here.
    // stdin.read_line(&mut buffer)?;
    // Ok(())
}

fn main() {
    let mut args = pico_args::Arguments::from_env();

    let exit_code = match args.subcommand() {
        Ok(Some(s)) => {
            if s.eq("set") {
                subcommand_set(args)
            } else if s.eq("monitor") {
                subcommand_monitor(args)
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
            eprintln!("error setting kbd: {}", e);
            1
        }
    };

    std::process::exit(exit_code);
}
