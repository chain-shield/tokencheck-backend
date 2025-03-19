# Rust Web Server

## Environment Variables

This application uses the following environment variables for configuration. Create a `.env` file in the root directory with these variables.

### Database Configuration
- `DATABASE_URL`: Connection string for PostgreSQL database
  - Example: `postgres://user@localhost:5432/rust_web_server`
- `MAX_CONNECTIONS`: Maximum number of database connections
  - Example: `10`

### Server Configuration
- `IP`: Server host address
  - Example: `127.0.0.1`
- `PORT`: Server port number
  - Example: `8080`
- `WORKERS`: Number of worker threads for the server
  - Example: `4`
- `CORS_ALLOWED_ORIGIN`: Allowed origin for CORS (defaults to "http://localhost:3000" if not specified)
  - Example: Not specified in sample .env (uses default)
- `ENABLE_CONSOLE_LOGGING`: Enable console logging (defaults to "true" if not specified)
  - Example: `true`

### JWT Authentication
- `JWT_SECRET`: Secret key for JWT token generation and validation
  - Example: `your_very_secure_and_long_random_secret_key` (use a strong random key in production)
- `JWT_EXPIRATION_HOURS`: Expiration time for JWT tokens in hours (defaults to "24" if not specified)
  - Example: `24`

### Logging
- `RUST_LOG`: Logging level for the application
  - Example: `debug`

### OAuth Providers
#### GitHub (Optional)
- `GITHUB_CLIENT_ID`: OAuth client ID from GitHub
  - Example: `Ov25lt5AatITKbBO3GJP`
- `GITHUB_CLIENT_SECRET`: OAuth client secret from GitHub
  - Example: `1t15dd307cfa19a311ef1602b47232a95bdff89c`
- `GITHUB_AUTH_URL`: GitHub authorization URL
  - Example: `https://github.com/login/oauth/authorize`
- `GITHUB_TOKEN_URL`: GitHub token URL
  - Example: `https://github.com/login/oauth/access_token`
- `GITHUB_REDIRECT_URI`: Redirect URI after GitHub authentication
  - Example: `http://localhost:8080/api/auth/oauth/github/callback`

#### Google (Optional)
- `GOOGLE_CLIENT_ID`: OAuth client ID from Google
  - Example: `your_google_client_id`
- `GOOGLE_CLIENT_SECRET`: OAuth client secret from Google
  - Example: `your_google_client_secret`
- `GOOGLE_AUTH_URL`: Google authorization URL
  - Example: `https://accounts.google.com/o/oauth2/auth`
- `GOOGLE_TOKEN_URL`: Google token URL
  - Example: `https://accounts.google.com/o/oauth2/token`
- `GOOGLE_REDIRECT_URI`: Redirect URI after Google authentication
  - Example: `http://localhost:8080/api/auth/oauth/google/callback`