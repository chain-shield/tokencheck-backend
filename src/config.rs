use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub jwt_config: JwtConfig,
    pub server_host: String,
    pub server_port: u16,
    pub num_workers: usize,
    pub cors_allowed_origin: String,
    pub console_logging_enabled: bool,
    pub github_client: ProviderClient,
}

#[derive(Clone, Debug)]
pub struct ProviderClient {
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub redirect_uri: String,
}

#[derive(Clone, Debug)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration_hours: i64,
}

impl JwtConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        JwtConfig {
            secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .expect("JWT_EXPIRATION_HOURS must be a valid number"),
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        Config {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            jwt_config: JwtConfig::from_env(),
            server_host: env::var("IP").expect("IP must be set"),
            server_port: env::var("PORT")
                .expect("PORT must be set")
                .parse()
                .expect("Failed to parse PORT as u16"),
            num_workers: env::var("WORKERS")
                .expect("WORKERS must be set")
                .parse()
                .expect("Failed to parse WORKERS as usize"),
            cors_allowed_origin: env::var("CORS_ALLOWED_ORIGIN")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            console_logging_enabled: env::var("ENABLE_CONSOLE_LOGGING")
                .unwrap_or_else(|_| "true".to_string())
                .to_lowercase()
                == "true",
            github_client: ProviderClient {
                client_id: env::var("GITHUB_CLIENT_ID").expect("GITHUB_CLIENT_ID must be set"),
                client_secret: env::var("GITHUB_CLIENT_SECRET").expect("GITHUB_CLIENT_SECRET must be set"),
                auth_url: env::var("GITHUB_AUTH_URL").expect("GITHUB_AUTH_URL must be set"),
                token_url: env::var("GITHUB_TOKEN_URL").expect("GITHUB_TOKEN_URL must be set"),
                redirect_uri: env::var("GITHUB_REDIRECT_URI").expect("GITHUB_REDIRECT_URI must be set"),
            }
        }
    }
}
