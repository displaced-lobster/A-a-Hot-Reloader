#[macro_use]
extern crate clap;
extern crate ctrlc;

#[macro_use]
extern crate slog;

extern crate aa;

use aa::create_logger;
use aa::watchers::Watcher;

use std::{env, process, thread, time};
use std::process::Command;

fn main() {
    ctrlc::set_handler(move || {
        process::exit(0);
    }).expect("Error setting SIGINT handler");

    let matches = clap_app!(aa =>
        (version: "0.2.0")
        (author: "Richard M. <scripts.richard@gmail.com>")
        (about: "A'a - a hot reloader to watch a directory or single file and execute a command when it is modified.")
        (@arg COMMAND: +required +multiple "The command to be executed")
        (@arg TARGET: -f --file +takes_value "A specific file to be watched")
        (@arg TIME: -t --time +takes_value "Set the time interval (in seconds) to check for file changes")
        (@arg verbose: -v --verbose +multiple "Prints additional output")
    ).get_matches();

    let log_level = match matches.occurrences_of("verbose") {
        0 => slog::Level::Error,
        1 => slog::Level::Info,
        2 => slog::Level::Debug,
        3 | _ => slog::Level::Trace,
    };

    let logger = create_logger(log_level);

    let mut watcher = if let Some(target) = matches.value_of("TARGET") {
        info!(logger, "Watching file '{}'", target);

        Watcher::file_watcher(&target)
    } else {
        if let Some(path) = env::current_dir().unwrap().to_str() {
            info!(logger, "Watching directory '{}'", path);

            Watcher::dir_watcher(&path)
        } else {
            panic!("Failed to get current working directory")
        }
    };

    let command: Vec<String> = values_t!(matches.values_of("COMMAND"), String).unwrap();
    let time = matches.value_of("TIME").unwrap_or("2");

    let check_interval = time::Duration::from_secs(time.parse::<u64>().unwrap());

    let (exec, args) = command.split_first().unwrap();

    info!(logger, "Checking on {} second interval", time);
    info!(logger, "On change, executing '{}'", exec);

    loop {
        thread::sleep(check_interval);

        if watcher.was_modified().unwrap() {
            debug!(logger, "Change detected");

            let output = Command::new(exec)
                            .args(args)
                            .output()
                            .expect("Failed to execute command");

            if !output.status.success() {
                error!(logger, "{}", String::from_utf8_lossy(&output.stderr));
            }
        }
    }
}
