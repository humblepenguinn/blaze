use colored::*;

pub fn print_error(error_msg: &str) {
    println!("{}: {}", "Error".red(), error_msg);
}
