pub fn log(str: &str) {
    log_proto(false, str);
}

pub fn succeed(str: &str) {
    log_proto(true, str);
}

fn log_proto(success: bool, str: &str) {
    let mut buf = "\u{001b}[33m----\u{001b}[0m  ".to_string();
    if success {
        buf.push_str("\u{001b}[32m");
    }
    buf.push_str(str);
    if success {
        buf.push_str("\u{001b}[0m");
    }

    println!("{}", buf.as_str());
}
