use chrono::DateTime;

pub const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
pub const AUTH_URL: &str = "https://accounts.gen.com/o/oauth2/auth";

#[derive(Debug, Clone)]
pub struct Token {
    pub access_token: String,
    pub token_type: String,
    pub expiry: Option<DateTime<chrono::Utc>>,
}

impl Token {
    pub fn value(&self) -> String {
        return format!("Bearer {}", self.access_token);
    }

    pub fn valid(&self) -> bool {
        !self.access_token.is_empty() && !self.expired()
    }

    fn expired(&self) -> bool {
        if self.expiry.is_none() {
            return false;
        }
        let now = chrono::Utc::now();
        let exp = self.expiry.unwrap() + chrono::Duration::seconds(-10);
        return now > exp;
    }
}
