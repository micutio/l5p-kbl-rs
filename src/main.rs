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

fn main() {
    let mut args = pico_args::Arguments::from_env();

    if args.contains(["-h", "--help"]) {
        println!("{}", msg::HELP_GENERAL);
        std::process::exit(0);
    }

    let exit_code = match args.subcommand() {
        Ok(Some(s)) => {
            if s.eq("set") {
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
            } else if s.eq("monitor") {
                // println!("{:#?}", led::parse_parameters(args));
                // let mut child = monitor::listen(
                //     "org.gnome.desktop.peripherals.touchpad",
                //     "send-events",
                //     |line| {
                //         let l = line.unwrap();
                //         if l.contains("enabled") {
                //             println!("KEYBOARD LIGHT ON");
                //         }
                //         if l.contains("disabled") {
                //             println!("KEYBOARD LIGHT OFF");
                //         }
                //     },
                // )
                // .unwrap();

                let domain = "org.gnome.desktop.peripherals.touchpad";
                let key = "send-events";
                let states = vec![]; // TODO: fill
                let mut monitor = monitor::Monitor::new(domain, key, states).unwrap();

                monitor.wait();
                match monitor.close().kill() {
                    Ok(_) => 0,
                    Err(_) => 1,
                }

            // alternative: wait for user input to terminate the program
            // let mut buffer = String::new();
            // let mut stdin = io::stdin(); // We get `Stdin` here.
            // stdin.read_line(&mut buffer)?;
            // Ok(())
            } else {
                eprintln!("unknown command: {}", s);
                1
            }
        }
        Ok(None) => {
            eprintln!("missing command");
            1
        }
        Err(e) => {
            eprintln!("error setting kbd: {}", e);
            1
        }
    };

    std::process::exit(exit_code);
}
