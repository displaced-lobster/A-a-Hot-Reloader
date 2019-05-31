#[macro_use]
extern crate clap;
extern crate ctrlc;

use std::{fs, io, thread, time};
use std::process;
use std::process::Command;

fn get_modified(target: &str) -> Result<time::SystemTime, io::Error> {
    let modified = fs::metadata(target)?
                    .modified()?;

    Ok(modified)
}

fn main() {
    ctrlc::set_handler(move || {
        println!("\nNo longer watching file");
        process::exit(0);
    }).expect("Error setting SIGINT handler");

    let matches = clap_app!(aa =>
        (version: "0.1")
        (author: "Richard M. <scripts.richard@gmail.com>")
        (about: "A'a - a hot reloader to watch a file and execute a command when it changes.")
        (@arg TARGET: +required "The file to be watched")
        (@arg COMMAND: +required "The command to be executed")
    ).get_matches();

    let target = matches.value_of("TARGET").unwrap();
    let command = matches.value_of("COMMAND").unwrap();
    let check_interval = time::Duration::from_secs(2);

    println!("Watching file '{}'", target);
    println!("On change will execute '{}'", command);

    let mut check_time = time::SystemTime::now();

    loop {
        thread::sleep(check_interval);

        let modified = get_modified(&target).unwrap();

        if modified > check_time {
            let _ = Command::new(command)
                        .output()
                        .expect("Failed to execute command");
        }

        check_time = time::SystemTime::now();
    }
}
