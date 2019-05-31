#[macro_use]
extern crate clap;
extern crate ctrlc;

use std::{fs, thread, time};
use std::process;
use std::process::Command;

fn get_modified(target: &str) -> Result<u64, String> {
    let modified = fs::metadata(target)
                    .unwrap()
                    .modified()
                    .unwrap()
                    .duration_since(time::SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

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
        (about: "Hot reloader")
        (@arg TARGET: +required "The file to be watched")
        (@arg COMMAND: +required "The command to be executed")
    ).get_matches();

    let target = matches.value_of("TARGET").unwrap();
    let command = matches.value_of("COMMAND").unwrap();
    let five_seconds = time::Duration::from_secs(5);

    println!("Watching file '{}'", target);
    println!("On change will execute '{}'", command);

    let mut last_modified = get_modified(&target).unwrap();

    loop {
        thread::sleep(five_seconds);

        let modified = get_modified(&target).unwrap();

        if modified > last_modified {
            last_modified = modified;

            let _ = Command::new(command)
                        .output()
                        .expect("Failed to execute command");
        }
    }
}
