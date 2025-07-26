use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter, SharedLogger, TermLogger,
    TerminalMode, WriteLogger, format_description,
};
use std::{fs::OpenOptions, path::PathBuf};

fn get_log_filepath() -> Option<PathBuf> {
    // Explicit environment variable to disable file logging
    if let Ok(env_var) = std::env::var("DISKBLOCK_DISABLE_FILE_LOG") {
        match env_var.to_lowercase().as_str() {
            "1" | "true" => {
                return None;
            }
            _ => (),
        }
    }

    // Explicit environment variable to set output filepath
    if let Ok(env_var) = std::env::var("DISKBLOCK_LOG_FILEPATH") {
        if env_var.is_empty() {
            return None;
        }

        return Some(PathBuf::from(env_var));
    }

    // Default: Log to a file next to the binary
    Some(
        std::env::current_exe()
            .unwrap()
            .as_path()
            .parent()
            .unwrap()
            .join("diskblock.log"),
    )
}

pub fn init_logger() -> () {
    let log_config = ConfigBuilder::new()
        .set_time_format_custom(format_description!(
            "[year]-[month]-[day] [hour]:[minute]:[second]"
        ))
        .set_time_offset_to_local()
        .unwrap()
        .build();

    let mut loggers: Vec<Box<dyn SharedLogger>> = vec![TermLogger::new(
        LevelFilter::Info,
        log_config.clone(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )];

    if let Some(filepath) = get_log_filepath() {
        loggers.push(WriteLogger::new(
            LevelFilter::Info,
            log_config.clone(),
            OpenOptions::new()
                .write(true)
                .append(true)
                .create(true)
                .open(filepath)
                .unwrap(),
        ))
    }

    CombinedLogger::init(loggers).unwrap();
}
