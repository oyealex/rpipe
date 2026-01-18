#[macro_export]
macro_rules! print_err {
    () => {};
    ($($arg:tt)*) => {
        if std::io::IsTerminal::is_terminal(&std::io::stderr()) {
            eprint!("\x1b[1;31m");
            eprint!($($arg)*);
            eprint!("\x1b[0m");
        } else {
            eprint!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! println_err {
    () => {};
    ($($arg:tt)*) => {
        if std::io::IsTerminal::is_terminal(&std::io::stderr()) {
            eprint!("\x1b[1;31m");
            eprint!($($arg)*);
            eprintln!("\x1b[0m");
        } else {
            eprintln!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! println_info {
    () => {};
    ($($arg:tt)*) => {
        if std::io::IsTerminal::is_terminal(&std::io::stderr()) {
            print!("\x1b[1;34m");
            print!($($arg)*);
            println!("\x1b[0m");
        } else {
            println!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! println_notice {
    () => {};
    ($($arg:tt)*) => {
        if std::io::IsTerminal::is_terminal(&std::io::stderr()) {
            print!("\x1b[35m");
            print!($($arg)*);
            println!("\x1b[0m");
        } else {
            println!($($arg)*);
        }
    };
}
