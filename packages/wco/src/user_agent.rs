/// User agent strings for HTTP requests
#[derive(Debug, Clone, Copy)]
pub enum UserAgent {
    /// PostmanRuntime user agent
    PostMan,
    /// Google Chrome user agent
    GoogleChrome,
}

impl UserAgent {
    /// Get the user agent string
    pub fn as_str(&self) -> &'static str {
        match self {
            UserAgent::PostMan => "PostmanRuntime/7.45.0",
            UserAgent::GoogleChrome => "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36",
        }
    }
}

impl AsRef<str> for UserAgent {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
