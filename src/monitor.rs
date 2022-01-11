//! Inspired by https://github.com/paulgb/interactive_process
//!
//! Possible things to monitor:
//! - keyboard layout
//!   -  gsettings get org.gnome.desktop.input-sources [sources|current]
//! - dark/light theme
//!
//! Structure:
//!  - [setting domain], [setting key], [setting value substring], [led parameters]
//!

use std::io::BufRead;

use crate::led;

pub(crate) struct Monitor {
    child_proc: std::process::Child,
}

impl Monitor {
    pub fn new(
        domain: &str,
        key: &str,
        states: Vec<(String, led::Parameters)>,
    ) -> std::io::Result<Self> {
        let mut child = match std::process::Command::new("gsettings")
            .arg("monitor")
            .arg(domain)
            .arg(key)
            .stdout(std::process::Stdio::piped())
            .spawn()
        {
            Ok(it) => it,
            Err(err) => return Err(err),
        };

        let stdout = child
            .stdout
            .take()
            .expect("Accessing stdout should never fail after passing Stdio::piped().");

        std::thread::spawn(move || {
            std::io::BufReader::new(stdout)
                .lines()
                .for_each(|line_result| {
                    if let Ok(line) = line_result {
                        for (val, params) in &states {
                            if line.contains(val) {
                                match led::set_led(params.clone()) {
                                    Ok(_) => {}
                                    Err(err) => eprintln!("error setting led: {}", err),
                                }
                                break;
                            }
                        }
                    }
                });
            println!("gsettings monitor terminated");
        });

        Ok(Monitor { child_proc: child })
    }

    pub fn wait(&mut self) {
        match self.child_proc.wait() {
            Ok(_) => {}
            Err(err) => {
                eprintln!("error parsing params: {}", err);
            }
        }
    }

    pub fn close(self) -> std::process::Child {
        self.child_proc
    }
}
