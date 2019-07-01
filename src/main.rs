#[macro_use]
extern crate clap;
extern crate ctrlc;

#[macro_use]
extern crate slog;

extern crate aa;

use aa::create_logger;
use aa::watchers::{Traversal, Watcher};

use std::{env, process};
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
        (@arg verbose: -v --verbose +multiple "Prints additional output")
        (@arg recursive: -r --recursive "Recursively watch the directory")
        (@arg FILE: -f --file +takes_value "A specific file to be watched")
        (@arg PATH: -p --path +takes_value "A path to be watched")
    ).get_matches();

    let log_level = match matches.occurrences_of("verbose") {
        0 => slog::Level::Error,
        1 => slog::Level::Info,
        2 => slog::Level::Debug,
        3 | _ => slog::Level::Trace,
    };

    let logger = create_logger(log_level);

    let mut watcher = if let Some(target) = matches.value_of("FILE") {
        info!(logger, "Watching file '{}'", target);

        Watcher::file_watcher(&target).unwrap()
    } else {
        let path = if let Some(path) = matches.value_of("PATH") {
            String::from(path)
        } else if let Some(path) = env::current_dir().unwrap().to_str() {
            String::from(path)
        } else {
            panic!("Failed to get path");
        };

        info!(logger, "Watching directory '{}'", path);

        let traversal = if matches.is_present("recursive") {
            Traversal::RECURSIVE
        } else {
            Traversal::HEURISTIC
        };

        Watcher::dir_watcher(&path, traversal).unwrap()
    };

    watcher.register_logger(logger.new(o!("watcher" => 1)));

    let command: Vec<String> = values_t!(matches.values_of("COMMAND"), String).unwrap();
    let (exec, args) = command.split_first().unwrap();

    info!(logger, "On change, executing '{}'", exec);

    while watcher.watch().unwrap() {
        info!(logger, "Change detected");

        let output = Command::new(exec)
                        .args(args)
                        .output()
                        .expect("Failed to execute command");

        if !output.status.success() {
            error!(logger, "{}", String::from_utf8_lossy(&output.stderr));
        }
    }
}
