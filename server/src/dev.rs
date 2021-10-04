////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Model;

enum Sentiment {
    Success,
    Failure,
}

////////////////////////////////////////////////////////////////////////////////
// Api  //
////////////////////////////////////////////////////////////////////////////////

impl Model {
    pub fn init() -> Model {
        Model
    }
}

impl Sentiment {
    pub fn terminal_code<'a>(&self) -> &'a str {
        match self {
            Sentiment::Success => "32",
            Sentiment::Failure => "31",
        }
    }
}

pub fn log(str: &str) {
    log_proto(None, str);
}

pub fn succeed(str: &str) {
    log_proto(Some(Sentiment::Success), str);
}

pub fn error(str: &str) {
    log_proto(Some(Sentiment::Failure), str);
}

fn log_proto(maybe_sentiment: Option<Sentiment>, str: &str) {
    let mut buf = "\u{001b}[33m----\u{001b}[0m  ".to_string();
    if let Some(ref sentiment) = maybe_sentiment {
        buf.push_str("\u{001b}[");
        buf.push_str(sentiment.terminal_code());
        buf.push_str("m");
    }

    buf.push_str(str);

    if let Some(_) = maybe_sentiment {
        buf.push_str("\u{001b}[0m");
    }

    println!("{}", buf.as_str());
}
