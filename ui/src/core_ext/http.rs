use seed::browser::fetch::FetchError;
use seed::browser::fetch::JsonError;

pub fn fetch_error_to_string(error: FetchError) -> String {
    match error {
        // FetchError::SerdeError(error) => error.to_string(),
        FetchError::DomException(_error) => "Dom Exception".to_string(),
        FetchError::PromiseError(error) => {
            let mut buf = String::new();

            buf.push_str("Promise Error : ");
            buf.push_str(format!("{:?}", error).as_str());
            buf
        }
        FetchError::NetworkError(error) => {
            let mut buf = String::new();

            buf.push_str("Network Error : ");
            buf.push_str(format!("{:?}", error).as_str());
            buf
        }
        FetchError::RequestError(error) => {
            let mut buf = String::new();

            buf.push_str("Request Error : ");
            buf.push_str(format!("{:?}", error).as_str());
            buf
        }
        FetchError::StatusError(status) => {
            let mut buf = String::new();

            buf.push_str("Code : ");
            buf.push_str(status.code.to_string().as_str());
            buf.push_str(", Message : ");
            buf.push_str(status.text.as_str());
            buf
        }
        FetchError::JsonError(sub_error) => {
            let js_value = match sub_error {
                JsonError::Serde(serde_error) => serde_error,
                JsonError::Parse(parse_error) => parse_error,
                JsonError::Stringify(stringify_error) => stringify_error,
            };

            js_value
                .as_string()
                .unwrap_or("Json error that could not be unwrapped".to_string())
        }
    }
}
