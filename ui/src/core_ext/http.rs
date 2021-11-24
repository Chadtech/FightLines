use seed::browser::fetch::FetchError;

pub fn fetch_error_to_string(error: FetchError) -> String {
    match error {
        FetchError::SerdeError(error) => error.to_string(),
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
    }
}
