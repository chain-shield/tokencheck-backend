use std::fmt;

use serde::{Deserialize, Serialize};

use super::error::{AppError, Res};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OAuthProvider {
    GitHub,
    Google,
    // Facebook,
}
impl OAuthProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            OAuthProvider::GitHub => "github",
            OAuthProvider::Google => "google",
            // OAuthProvider::Facebook => "facebook",
        }
    }
    pub fn from_str(s: &str) -> Res<Self> {
        match s {
            "github" => Ok(OAuthProvider::GitHub),
            "google" => Ok(OAuthProvider::Google),
            // "facebook" => Ok(OAuthProvider::Facebook),
            ps => Err(AppError::Internal(format!(
                "Invalid OAuth provider: {}",
                ps
            ))),
        }
    }
    pub fn get_scopes(&self) -> Vec<&'static str> {
        match self {
            OAuthProvider::GitHub => vec!["user:email"],
            OAuthProvider::Google => vec!["email profile"],
            // OAuthProvider::Facebook => vec!["email"],
        }
    }
}
impl fmt::Display for OAuthProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
