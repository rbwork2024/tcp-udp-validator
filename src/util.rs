pub fn print_and_log(msg: &str, print: bool, level: log::Level) {
    log::log!(
        level,
        "[{}] {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        msg
    );
    if print {
        println!(
            "{} [{}] {}",
            level,
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            msg
        );
    }
}
