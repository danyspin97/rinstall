use colored::Colorize;
use log::{Level, Metadata, Record};

pub struct SimpleLogger {
    pub quiet: bool,
}

impl log::Log for SimpleLogger {
    fn enabled(
        &self,
        metadata: &Metadata,
    ) -> bool {
        metadata.level() <= Level::Warn || !self.quiet
    }

    fn log(
        &self,
        record: &Record,
    ) {
        if self.enabled(record.metadata()) {
            match record.level() {
                Level::Error => eprintln!("{}: {}", "ERROR".red().bold(), record.args()),
                Level::Warn => eprintln!("{}: {}", "WARNING".yellow().bold(), record.args()),
                Level::Info => println!("{}", record.args()),
                Level::Debug | Level::Trace => unreachable!(),
            }
        }
    }

    fn flush(&self) {}
}
