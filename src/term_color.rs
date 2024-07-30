pub enum Color {
    Green,
    Red,
    Yellow,
    Blue,
}

/// Given a string, print a colored version of it to the console.
pub fn colored(string: &str, color: Color) -> String {
    // the left hand side of the color swap, change to a specific color.
    let lhs = format!(
        "\x1b[{}m",
        match color {
            Color::Green => 92,
            Color::Yellow => 93,
            Color::Blue => 94,
            Color::Red => 91,
        }
    );

    // the right hand side of the swap, reset the color back to normal.
    let rhs = "\x1b[0m";

    // put the string inbetween the left and right hand side.
    format!("{lhs}{string}{rhs}")
}

#[macro_export]
macro_rules! log_warn {
    ($($args:tt)*) => {
        // format the string with var args.
        let string = std::fmt::format(format_args!($($args)*));
        // color the string according to the macro.
        println!("{}: {string}", colored("warning", Color::Yellow));
    }
}

#[macro_export]
macro_rules! log_error {
    ($($args:tt)*) => {
        // format the string with var args.
        let string = std::fmt::format(format_args!($($args)*));
        // color the string according to the macro.
        println!("{}: {string}", colored("error", Color::Red));
    }
}

#[macro_export]
macro_rules! log_success {
    ($($args:tt)*) => {
        // format the string with var args.
        let string = std::fmt::format(format_args!($($args)*));
        // color the string according to the macro.
        println!("{}: {string}", colored("success", Color::Green));
    }
}
