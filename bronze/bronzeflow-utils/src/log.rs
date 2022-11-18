#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        println!("info: {}", format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        println!("warn: {}", format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        eprintln!("warn: {}", format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{
        println!("debug: {}", format!($($arg)*));
    }};
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_log() {
        info!("hello, this is {} log", "info");
        warn!("hello, this is {} log", "warn");
        error!("hello, this is {} log", "error");
        error!("hello, this is {} log", "debug");
    }
}
