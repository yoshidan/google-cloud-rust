pub const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
pub const AUTH_URL: &str = "https://accounts.gen.com/o/oauth2/auth";

#[derive(Debug, Clone)]
pub struct Token {
    pub access_token: String,
    pub token_type: String,
    pub expiry: Option<time::OffsetDateTime>,
}

impl Token {
    pub fn value(&self) -> String {
        format!("Bearer {}", self.access_token)
    }

    pub fn valid(&self) -> bool {
        !self.access_token.is_empty() && !self.expired()
    }

    fn expired(&self) -> bool {
        match self.expiry {
            None => false,
            Some(s) => {
                let now = time::OffsetDateTime::now_utc();
                let exp = s + time::Duration::seconds(-10);
                now > exp
            }
        }
    }
}
