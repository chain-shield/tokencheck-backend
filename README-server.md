# Rust Web Server

## OAuth Integration Instructions

> [!WARNING]
> Currently, only GitHub and Google OAuth are fully implemented and tested.
> Other providers may require additional configuration or may not be fully functional.

## OAuth Integration Instructions

1. Redirect the user's browser to the `/api/auth/oauth/{provider}` endpoint on the server. Replace `{provider}` with the name of the OAuth provider (e.g., `google`, `github`).
```javascript
const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080/api';
window.location.href = `${API_BASE_URL}/auth/oauth/${provider}`;
```

2. Setup `WEB_APP_AUTH_CALLBACK_URL` variable in `.env` file in Rust server. The server will redirect to that URL after it's done with authentication.
```
WEB_APP_AUTH_CALLBACK_URL=http://localhost:3000/auth/callback
```

3. In the component that handles this authentication callback, read cookies `token` and `user`. If not present, make a call to `/api/session` which retrieves token and user data, and returns it as a JSON body in the response.
```javascript
import Cookies from 'js-cookie';
import { useEffect } from 'react';
import { useRouter } from 'next/navigation';

export default function OAuthCallbackPage() {
    const router = useRouter();
    useEffect(() => {
        async function processCallback() {
            try {
                let token = Cookies.get('token');
                let userJson = Cookies.get('user');

                if (!token || !userJson) {
                    const response = await fetch('http://localhost:8080/api/session', {
                        credentials: 'include',
                        method: 'GET'
                    });
                    if (!response.ok) {
                        throw new Error("API failed")
                    }
                    const data = await response.json();
                    token = data.token;
                    userJson = JSON.stringify(data.user);
                }

                if (!token || !userJson) {
                    // error: "Missing authentication data";
                    return;
                }

                const user = JSON.parse(userJson);
                // you can store token and user in localStorage or in a cookie here
                
                // redirect to home page or user dashboard
                router.push('/');
            } catch (error) {
                // error: ('Failed to process OAuth callback:', error);
            }
        }
        processCallback();
    }, [router]);
}
```

## Environment Variables

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
- `{provider}_CLIENT_ID`: OAuth client ID
  - Example: `Ov25lt5AatITKbBO3GJP`
- `{provider}_CLIENT_SECRET`: OAuth client secret
  - Example: `1t15dd307cfa19a311ef1602b47232a95bdff89c`
- `{provider}_AUTH_URL`: Authorization URL
  - Example: `https://github.com/login/oauth/authorize`
- `{provider}_TOKEN_URL`: Token URL
  - Example: `https://github.com/login/oauth/access_token`
- `{provider}_REDIRECT_URI`: Redirect URI after authentication process
  - Example: `http://localhost:8080/api/auth/oauth/github/callback`

Replace `{provider}` with `GITHUB`/`GOOGLE`/`FACEBOOK`/`APPLE`/`TWITTER`