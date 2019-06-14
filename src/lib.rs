#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

use slog::Drain;

pub mod watchers;

pub fn create_logger(log_level: slog::Level) -> slog::Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let drain = slog::LevelFilter::new(drain, log_level).fuse();

    slog::Logger::root(drain, o!())
}
