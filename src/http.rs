use ext_php_rs::prelude::*;

#[php_class]
#[derive(Debug, Default)]
pub struct HttpReq {
    #[php(prop)]
    pub method: String,
    #[php(prop)]
    pub path: String,
    #[php(prop)]
    pub headers: Vec<String>, // Simplified for demo: "Key: Value"
    #[php(prop)]
    pub body: String,
}

#[php_class]
pub struct HttpParser;

#[php_impl]
impl HttpParser {
    pub fn parse_request(data: String) -> Option<HttpReq> {
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut req = httparse::Request::new(&mut headers);

        match req.parse(data.as_bytes()) {
            Ok(httparse::Status::Complete(body_start)) => {
                let mut r = HttpReq::default();
                r.method = req.method.unwrap_or("").to_string();
                r.path = req.path.unwrap_or("").to_string();
                
                for h in req.headers {
                    let name = h.name;
                    let value = std::str::from_utf8(h.value).unwrap_or("");
                    r.headers.push(format!("{}: {}", name, value));
                }

                r.body = data[body_start..].to_string();
                Some(r)
            }
            _ => None, // Incomplete or Error
        }
    }
}
