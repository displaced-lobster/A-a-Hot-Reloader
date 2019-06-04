#[macro_use]
extern crate clap;
extern crate ctrlc;

#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

use std::{fs, io, thread, time};
use std::process;
use std::process::Command;

use slog::Drain;

fn get_modified(target: &str) -> Result<time::SystemTime, io::Error> {
    let modified = fs::metadata(target)?
                    .modified()?;

    Ok(modified)
}

fn create_logger(log_level: slog::Level) -> slog::Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let drain = slog::LevelFilter::new(drain, log_level).fuse();

    slog::Logger::root(drain, o!())
}

fn main() {
    ctrlc::set_handler(move || {
        process::exit(0);
    }).expect("Error setting SIGINT handler");

    let matches = clap_app!(aa =>
        (version: "0.1")
        (author: "Richard M. <scripts.richard@gmail.com>")
        (about: "A'a - a hot reloader to watch a file and execute a command when it changes.")
        (@arg TARGET: +required "The file to be watched")
        (@arg COMMAND: +required +multiple "The command to be executed")
        (@arg TIME: -t +takes_value "Set time time interval (in seconds) to check for file changes")
        (@arg verbose: -v +multiple "Set the verbosity")
    ).get_matches();

    let log_level = match matches.occurrences_of("verbose") {
        0 => slog::Level::Info,
        1 => slog::Level::Debug,
        2 | _ => slog::Level::Trace,
    };

    let logger = create_logger(log_level);

    let target = matches.value_of("TARGET").unwrap();
    let command: Vec<String> = values_t!(matches.values_of("COMMAND"), String).unwrap();
    let time = matches.value_of("TIME").unwrap_or("2");

    let check_interval = time::Duration::from_secs(time.parse::<u64>().unwrap());

    let (exec, args) = command.split_first().unwrap();

    info!(logger, "Watching target '{}'", target);
    info!(logger, "Checking on {} second interval", time);
    info!(logger, "On change, executing '{}'", exec);

    let mut check_time = time::SystemTime::now();

    loop {
        thread::sleep(check_interval);

        let modified = get_modified(&target).unwrap();

        if modified > check_time {
            debug!(logger, "Change detected");

            let output = Command::new(exec)
                            .args(args)
                            .output()
                            .expect("Failed to execute command");

            if !output.status.success() {
                error!(logger, "{}", String::from_utf8_lossy(&output.stderr));
            }
        }

        check_time = time::SystemTime::now();
    }
}
