# Redis Examples

This directory contains examples demonstrating how to use the async Redis client with TLS support.

## Examples

### 1. basic.php
Basic Redis operations including:
- String operations (GET, SET)
- Counters (INCR, DECR)
- Lists (LPUSH, RPUSH, LPOP, LRANGE)
- Hashes (HSET, HGET, HGETALL)
- Sets (SADD, SMEMBERS, SISMEMBER)
- Key management (EXISTS, DEL, TTL, EXPIRE)

**Run:**
```bash
php -d extension=target/release/libasync_php.dylib examples/redis/basic.php
```

### 2. tls_connection.php
TLS/SSL connection examples including:
- Secure TLS connection with certificate verification
- Insecure mode (skip certificate verification) for testing
- URL-based connection (`rediss://` scheme)
- Authentication with TLS

**Run:**
```bash
php -d extension=target/release/libasync_php.dylib examples/redis/tls_connection.php
```

## TLS Connection Methods

### Method 1: Using `connectTls()`

```php
$redis = new \Async\Kernel\Redis\Client();

// Secure connection (verifies certificates)
$redis->connectTls('redis.example.com', 6380)->await();

// Insecure mode (skip certificate verification - testing only!)
$redis->connectTls('localhost', 6380, 0.0, null, null, 0.0, true)->await();
```

### Method 2: Using `connect()` with URL

```php
$redis = new \Async\Kernel\Redis\Client();

// Secure TLS connection
$redis->connect('rediss://redis.example.com:6380')->await();

// Insecure mode (skip certificate verification)
$redis->connect('rediss://localhost:6380#insecure')->await();

// With authentication
$redis->connect('rediss://username:password@redis.example.com:6380')->await();
```

## TLS Configuration

### Server Setup

To test TLS connections locally, you need a Redis server with TLS enabled:

1. **Generate self-signed certificates** (for testing):
```bash
# Generate private key
openssl genrsa -out redis.key 2048

# Generate certificate
openssl req -new -x509 -key redis.key -out redis.crt -days 365 \
  -subj "/CN=localhost"

# Generate CA certificate (optional)
openssl req -new -x509 -key redis.key -out ca.crt -days 365 \
  -subj "/CN=Redis CA"
```

2. **Configure Redis** (`redis.conf`):
```conf
# Enable TLS
port 0
tls-port 6380
tls-cert-file /path/to/redis.crt
tls-key-file /path/to/redis.key
tls-ca-cert-file /path/to/ca.crt

# Optional: require client certificates
# tls-auth-clients yes
```

3. **Start Redis with TLS**:
```bash
redis-server /path/to/redis.conf
```

### Client Configuration

The client supports two TLS backends via Cargo features:

- **rustls** (default): Pure Rust TLS implementation
  - Feature: `tokio-rustls-comp`
  - Recommended for most use cases

- **native-tls**: System native TLS (OpenSSL/Secure Transport)
  - Feature: `tokio-native-tls-comp`
  - Use if you need OS-native certificate stores

Current configuration in `Cargo.toml`:
```toml
redis = { version = "1.0", features = ["tokio-comp", "connection-manager", "aio", "tokio-rustls-comp"] }
```

## Connection URL Formats

### Standard Redis (no TLS)
```
redis://[username][:password]@[host]:[port][/database]
```

Examples:
- `redis://127.0.0.1:6379`
- `redis://localhost:6379/0`
- `redis://user:pass@redis.example.com:6379`

### Redis with TLS
```
rediss://[username][:password]@[host]:[port][/database][#insecure]
```

Examples:
- `rediss://redis.example.com:6380`
- `rediss://localhost:6380#insecure` (skip cert verification)
- `rediss://default:password@redis.example.com:6380/0`

## Security Notes

### Certificate Verification

**Production environments:**
- Always use proper SSL/TLS certificates
- Never use `#insecure` mode in production
- Use certificates from a trusted CA
- Keep certificates and keys secure

**Development/Testing:**
- You can use self-signed certificates with `#insecure` flag
- This skips certificate verification
- Useful for local testing only

### Authentication

Always use authentication in production:
```php
// URL format
$redis->connect('rediss://username:password@host:6380')->await();
```

Configure Redis with:
```conf
requirepass your-strong-password
# Or use ACL (Redis 6+)
aclfile /path/to/users.acl
```

## Common Issues

### Certificate Verification Failed
```
Error: Failed to connect with TLS: SSL error
```

Solutions:
1. Use `#insecure` for testing (not production!)
2. Install proper CA certificates
3. Use valid domain name matching certificate CN/SAN

### Connection Timeout
```
Error: Failed to connect: Connection timeout
```

Solutions:
1. Check if Redis server is running with TLS enabled
2. Verify port number (6380 for TLS by default)
3. Check firewall rules

### Wrong Port
```
Error: Failed to connect: Connection refused
```

Solutions:
1. Ensure using TLS port (typically 6380, not 6379)
2. Check Redis configuration (`tls-port` setting)
3. Verify Redis is listening on the correct interface

## API Reference

### Client Methods

#### `connect(host, port, timeout, reserved, retry_interval, read_timeout)`
Connect to Redis server. Supports both `redis://` and `rediss://` URL schemes.

#### `connectTls(host, port, timeout, reserved, retry_interval, read_timeout, insecure)`
Connect to Redis server with TLS encryption.
- `insecure`: Set to `true` to skip certificate verification (default: `false`)

#### `close()`
Close the Redis connection.

#### `ping([message])`
Ping the server. Optionally with a message.

#### String Operations
- `set(key, value, [timeout])` - Set string value with optional expiration
- `get(key)` - Get string value
- `incr(key)` - Increment integer value
- `incrBy(key, value)` - Increment by specific value
- `decr(key)` - Decrement integer value
- `decrBy(key, value)` - Decrement by specific value

#### Key Management
- `del(keys)` - Delete one or more keys
- `exists(key)` - Check if key exists
- `expire(key, seconds)` - Set expiration on key
- `ttl(key)` - Get time to live for key

#### List Operations
- `lPush(key, values)` - Push to list (left)
- `rPush(key, values)` - Push to list (right)
- `lPop(key)` - Pop from list (left)
- `rPop(key)` - Pop from list (right)
- `lLen(key)` - Get list length
- `lRange(key, start, stop)` - Get list range

#### Hash Operations
- `hSet(key, field, value)` - Set hash field
- `hGet(key, field)` - Get hash field
- `hGetAll(key)` - Get all hash fields and values
- `hDel(key, fields)` - Delete hash fields
- `hExists(key, field)` - Check if hash field exists

#### Set Operations
- `sAdd(key, members)` - Add members to set
- `sMembers(key)` - Get all set members
- `sIsMember(key, member)` - Check if member exists in set
- `sRem(key, members)` - Remove members from set

#### Error Handling
- `getLastError()` - Get last error message
- `clearLastError()` - Clear last error

## Resources

- [Redis TLS Documentation](https://redis.io/docs/management/security/encryption/)
- [redis-rs GitHub](https://github.com/redis-rs/redis-rs)
- [Redis Security Best Practices](https://redis.io/docs/management/security/)
