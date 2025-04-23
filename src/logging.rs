use std::sync::{Once, OnceLock};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;

/**
Define the message structure sent over the channel
- Everything needs to be public that the macros can construct these from other modules
*/
pub struct LogMessage {
    pub level: Level,
    pub message: String,
    pub location: &'static std::panic::Location<'static>,
}

static MIN_LEVEL: OnceLock<Level> = OnceLock::new();
static LOG_CHANNEL_SENDER: OnceLock<mpsc::Sender<LogMessage>> = OnceLock::new();
static SPAWN_WORKER_ONCE: Once = Once::new();

/**
Define acceptable log levels
*/
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Level {
    Debug,
    Info,
    Okay,
    Warning,
    Fail,
}

/**
Log level implementation
*/
impl Level {
    pub fn as_str(&self) -> &'static str {
        match self {
            Level::Debug => "DBUG",
            Level::Info => "INFO",
            Level::Okay => "OKAY",
            Level::Warning => "WARN",
            Level::Fail => "FAIL",
        }
    }

    pub fn color_code(&self) -> &'static str {
        match self {
            Level::Debug => "\x1b[35m",   // Purple
            Level::Info => "\x1b[34m",    // Blue
            Level::Okay => "\x1b[32m",    // Green
            Level::Warning => "\x1b[33m", // Orange/Yellow
            Level::Fail => "\x1b[31m",    // Red
        }
    }
}

/**
Helper function to initialize the logging system
@param level The minimum level to log
*/
pub fn init(level: Level) {
    // Set the minimum level safely
    let _ = MIN_LEVEL.set(level);
    // Ensure the worker thread is started (if not already)
    ensure_worker_started();
}

/**
Helper function to check if logging is enabled for a given level
@param level The level to check
@return Boolean indicating if logging is enabled for the given level, false otherwise
*/
pub fn log_enabled(level: Level) -> bool {
    // Read the minimum level safely, defaulting to Info if not initialized
    level >= *MIN_LEVEL.get().unwrap_or(&Level::Info)
}

/**
Helper function to get and format timestamps
@return String containing the formatted timestamp
*/
pub fn format_timestamp() -> String {
    // Get the current time
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    // Convert to seconds and calculate date/time components
    let total_secs = now.as_secs();
    let (secs, mins, hours) = (
        total_secs % 60,
        (total_secs / 60) % 60,
        (total_secs / 3600) % 24,
    );

    // Calculate date
    let days_since_epoch = total_secs / 86400;

    // Very simple date calculation
    let (mut year, mut month, mut day) = (1970, 1, 1);
    let mut days_remaining = days_since_epoch;

    // Calculate years
    for y in 1970.. {
        let days_in_year = if is_leap_year(y) { 366 } else { 365 };
        if days_remaining < days_in_year {
            year = y;
            break;
        }
        days_remaining -= days_in_year;
    }

    // Calculate month and day
    let days_in_month = [
        31,
        if is_leap_year(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    for (m, &days) in days_in_month.iter().enumerate() {
        if days_remaining < days {
            month = m as u64 + 1;
            day = days_remaining + 1;
            break;
        }
        days_remaining -= days;
    }

    // Format the timestamp
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        year, month, day, hours, mins, secs
    )
}

/**
Helper function to check if a year is a leap year
@param year: The year to check
@return: True if the year is a leap year, false otherwise
*/
fn is_leap_year(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/**
Initialize the channel and spawn the worker thread
*/
fn ensure_worker_started() {
    SPAWN_WORKER_ONCE.call_once(|| {
        // Create bounded channel
        let (tx, mut rx) = mpsc::channel::<LogMessage>(1024);
        // Store the sender end
        if LOG_CHANNEL_SENDER.set(tx).is_err() {
            // Handle error
            eprintln!("Logger worker already initialized.");
            return;
        }

        // Spawn a background thread to handle actual logging
        thread::spawn(move || {
            // This thread owns the receiver
            while let Some(log_entry) = rx.blocking_recv() {
                let timestamp = format_timestamp();
                let color_code = log_entry.level.color_code();
                let reset_code = "\x1b[0m";

                eprintln!(
                    "[{}] - {}[{}]{} - [{}]\t| {}",
                    timestamp,
                    color_code,
                    log_entry.level.as_str(),
                    reset_code,
                    log_entry.location,
                    log_entry.message
                );
            }
        });
    });
}

/**
Helper function to get the sender, initialize worker if needed
@return: Sender
*/
pub fn get_sender() -> Option<&'static mpsc::Sender<LogMessage>> {
    // Ensure worker is started on first attempt to get sender
    ensure_worker_started();
    // Retrieve sender
    LOG_CHANNEL_SENDER.get()
}

/**
Macro rules for easy access to logging functions from other modules
*/
// Main macro for the logging functions
#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)+) => {{
        // Check level first to avoid unnecessary work
        if $crate::logging::log_enabled($level) {
            // Get the sender, potentially initializing the worker thread
            if let Some(sender) = $crate::logging::get_sender() {
                let location = std::panic::Location::caller();
                let message = format!($($arg)+);
                // Construct the LogMessage - fields are now accessible
                let log_entry = $crate::logging::LogMessage {
                    level: $level,
                    message,
                    location,
                };

                // Use try_send for non-blocking behavior.
                match sender.try_send(log_entry) {
                    Ok(_) => {} // Message sent successfully
                    Err(_e) => { // Handle error with bad blocking log message
                        eprintln!("Warning: Log message dropped (channel full or closed)");
                    }
                }
            }
            else { eprintln!("Logging system failed to initialize."); } // Handle initialization failure
        }
    }};
}

// Individual log level macros for easy calling from other modules
#[macro_export]
macro_rules! dbug {
    ($($arg:tt)+) => { $crate::log!($crate::logging::Level::Debug, $($arg)+) };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => { $crate::log!($crate::logging::Level::Info, $($arg)+) };
}

#[macro_export]
macro_rules! okay {
    ($($arg:tt)+) => { $crate::log!($crate::logging::Level::Okay, $($arg)+) };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)+) => { $crate::log!($crate::logging::Level::Warning, $($arg)+) };
}

#[macro_export]
macro_rules! fail {
    ($($arg:tt)+) => { $crate::log!($crate::logging::Level::Fail, $($arg)+) };
}
