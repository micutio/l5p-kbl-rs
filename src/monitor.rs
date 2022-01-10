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

/// Listen to a gsettings key
pub fn listen<T>(key: &str, val: &str, line_callback: T) -> std::io::Result<std::process::Child>
where
    T: Fn(std::io::Result<String>) + Send + 'static,
{
    let mut child = match std::process::Command::new("gsettings")
        .arg("monitor")
        .arg(key)
        .arg(val)
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
        for line in std::io::BufReader::new(stdout).lines() {
            line_callback(line);
        }
        println!("gsettings monitor terminated");
    });

    Ok(child)
}
