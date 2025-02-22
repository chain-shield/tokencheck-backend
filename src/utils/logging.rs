//! Logging utilities for the application.
//!
//! This module initializes and configures logging using the [`fern`] library and the [`colored`] crate
//! for enhanced, colored log output. Log messages are written both to standard output and to a file
//! named `"snipper.log"` with a custom format that includes timestamps, log targets, and colored log levels.

use colored::*;
use std::fs::File;

/// Initializes the logger for the application.
///
/// This function configures logging by creating a log file ("snipper.log") and setting up a custom
/// log formatting using [`fern::Dispatch`]. The log format includes the current time, the log target,
/// the colored log level, and the message.
///
/// The logging level is set to **Debug** globally, but logging is disabled for the `"ethers_providers"`
/// and `"hyper"` targets to reduce unnecessary verbosity.
///
/// # Errors
///
/// Returns a [`fern::InitError`] if any error occurs during the log file creation or when applying
/// the logging configuration.
///
/// # Examples
///
/// ```
/// use crate::utils::logging::setup_logger;
/// use log::info;
///
/// fn main() -> Result<(), fern::InitError> {
///     setup_logger()?;
///     info!("Logger initialized successfully.");
///     // ... rest of the application logic ...
///     Ok(())
/// }
/// ```
pub fn setup_logger() -> Result<(), fern::InitError> {
    // Create the log file "snipper.log". If file creation fails, propagate the error.
    File::create("snipper.log").map_err(fern::InitError::Io)?;

    // Configure the logger using fern::Dispatch.
    fern::Dispatch::new()
        // Apply a custom log format.
        .format(|out, message, record| {
            // Select a color based on the log level.
            let color = match record.level() {
                log::Level::Info => "green",
                log::Level::Warn => "yellow",
                log::Level::Error => "red",
                log::Level::Debug => "magenta",
                log::Level::Trace => "bright black",
            };
            // Write the formatted log message including timestamp, target, colored level, and the log message.
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%H:%M:%S]"),
                record.target(),
                record.level().to_string().color(color),
                message
            ))
        })
        // Set the global logging level to Debug.
        .level(log::LevelFilter::Debug)
        // Disable logging for specific libraries to minimize noise.
        .level_for("ethers_providers", log::LevelFilter::Off)
        .level_for("hyper", log::LevelFilter::Off)
        // Write log messages to the standard output.
        .chain(std::io::stdout())
        // Also write log messages to the "snipper.log" file.
        .chain(fern::log_file("snipper.log")?)
        // Finalize the logger initialization.
        .apply()?;
    Ok(())
}
