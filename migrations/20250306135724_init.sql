-- USERS
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(100) NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
-- ITEMS
CREATE TABLE items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(64) NOT NULL
);
-- LOGS
CREATE TABLE request_logs (
    id SERIAL PRIMARY KEY,
    timestamp TIMESTAMP NOT NULL,
    method VARCHAR(10) NOT NULL,
    path TEXT NOT NULL,
    status_code INTEGER NOT NULL,
    params JSONB,
    payload JSONB
);
-- SUBSCRIPTIONS
CREATE TABLE subscription_tiers (			
    id SERIAL PRIMARY KEY,			
    name VARCHAR(50) NOT NULL UNIQUE,			
    daily_limit INT NOT NULL,			
    monthly_limit INT NOT NULL,			
    rate_limit_per_second INT NOT NULL,			
    features VARCHAR(64) NOT NULL,			
    price_monthly DECIMAL(10,2)	NOT NULL		
);		
INSERT INTO subscription_tiers (name, daily_limit, monthly_limit, rate_limit_per_second, features, price_monthly) 
    VALUES ('Free', 100, 1000, 5, 'Basic API', 0.00);
INSERT INTO subscription_tiers (name, daily_limit, monthly_limit, rate_limit_per_second, features, price_monthly) 
    VALUES ('Basic', 1000, 10000, 10, 'All APIs', 29.99);
INSERT INTO subscription_tiers (name, daily_limit, monthly_limit, rate_limit_per_second, features, price_monthly) 
    VALUES ('Pro', 10000, 100000, 20, 'All APIs + Faster', 199.99);
INSERT INTO subscription_tiers (name, daily_limit, monthly_limit, rate_limit_per_second, features, price_monthly) 
    VALUES ('Enterprise', 100000, 1000000, 50, 'All APIs + Faster + Support', 999.99);	
-- USER SUBSCRIPTIONS
CREATE TABLE user_subscriptions (			
    user_id UUID REFERENCES users(id) NOT NULL,			
    tier_id INT REFERENCES subscription_tiers(id) NOT NULL,			
    start_date TIMESTAMP NOT NULL,						
    subscription_status VARCHAR(20) NOT NULL	
);			