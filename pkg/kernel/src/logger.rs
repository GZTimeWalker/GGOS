use log::{LevelFilter, Metadata, Record};

pub fn initialize() {
    static LOGGER: Logger = Logger;
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(match option_env!("LOG") {
        Some("error") => LevelFilter::Error,
        Some("warn") => LevelFilter::Warn,
        Some("info") => LevelFilter::Info,
        Some("debug") => LevelFilter::Debug,
        Some("trace") => LevelFilter::Trace,
        _ => LevelFilter::Info,
    });
}

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= metadata.level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            match record.level() {
                log::Level::Error => println_warn!("[E] {}", record.args()),
                log::Level::Warn => println_warn!("[!] {}", record.args()),
                log::Level::Info => println!("[+] {}", record.args()),
                log::Level::Debug => println!("[D] {}", record.args()),
                log::Level::Trace => println!("[T] {}", record.args()),
            }
        }
    }

    fn flush(&self) {}
}
