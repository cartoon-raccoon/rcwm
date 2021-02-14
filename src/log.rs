#[cfg(debug_assertions)]
macro_rules! debug {
    ($fmt:expr) => {
        (println!(concat!("[DEBUG] ", $fmt)));
    };
    ($fmt:expr, $($arg:tt)*) => {
        (println!(concat!("[DEBUG] ", $fmt), $($arg)*));
    };
}

#[cfg(not(debug_assertions))]
macro_rules! debug {
    ($fmt:expr) => {};
    ($fmt:expr, $($arg:tt)*) => {};
}

macro_rules! info {
    ($fmt:expr) => {
        (println!(concat!("[*] ", $fmt)));
    };
    ($fmt:expr, $($arg:tt)*) => {
        (println!(concat!("[*] ", $fmt), $($arg)*));
    };
}

macro_rules! warn {
    ($fmt:expr) => {
        (panic!(println!(concat!("[!] ", $fmt))));
    };
    ($fmt:expr, $($arg:tt)*) => {
        (panic!(println!(concat!("[!] ", $fmt), $($arg)*)));
    };
}

macro_rules! fatal {
    ($fmt:expr) => {
        (panic!(println!(concat!("[FATAL] ", $fmt))));
    };
    ($fmt:expr, $($arg:tt)*) => {
        (panic!(println!(concat!("[FATAL] ", $fmt), $($arg)*)));
    };
}

macro_rules! error {
    ($fmt:expr) => {
        (println!(concat!("[ERROR] ", $fmt)));
    };
    ($fmt:expr, $($arg:tt)*) => {
        (println!(concat!("[ERROR] ", $fmt), $($arg)*));
    };
}

