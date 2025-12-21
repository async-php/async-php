# Redis Examples for async-php

This directory contains examples demonstrating how to use the async-php Redis client with various connection modes and operations.

## Overview

The async-php Redis client provides a drop-in replacement for the native PHP Redis extension, enabling asynchronous Redis operations. It hooks the standard `Redis` class, so you can use familiar Redis methods in an async context.

## Features

- **Async Operations**: All Redis operations are non-blocking
- **TLS/SSL Support**: Connect to Redis over encrypted TLS connections
- **Password Authentication**: Support for password-protected Redis servers
- **Standard API**: Compatible with PHP Redis extension API
- **Multiple Data Types**: Strings, Lists, Hashes, Sets, and more
- **Connection Pooling**: Efficient connection management (via URL format)

## Examples

### 1. basic.php - Basic Redis Operations

Demonstrates fundamental Redis operations including:
- Connecting to a password-protected Redis server
- String operations (SET, GET, SETEX)
- Counter operations (INCR, DECR)
- List operations (LPUSH, RPUSH, LPOP, RPOP)
- Hash operations (HSET, HGET, HGETALL)
- Set operations (SADD, SMEMBERS)
- Key management (EXISTS, EXPIRE, TTL, DEL)

**Usage:**
```bash
# Start Redis (with password)
podman run --rm -it -p 6379:6379 redis:alpine redis-server --requirepass "123456"

# Run the example from project root
php -d extension=target/release/libasync_php.dylib examples/redis/basic.php
```

### 2. tls_connection.php - TLS/SSL Encrypted Connections

Comprehensive example covering Redis operations over TLS connection, including:
- TLS connection with password authentication
- String operations over TLS
- Counter operations
- List operations
- Hash operations
- Set operations
- Key management
- Bulk operations and cleanup
- Error handling

**Usage:**
```bash
# Start Redis with TLS
./examples/redis/start-redis-tls.sh

# Run the example from project root
php -d extension=target/release/libasync_php.dylib examples/redis/tls_connection.php
```

## Setup Instructions

### Regular Redis Server (No Password)

```bash
podman run --rm -it -p 6379:6379 redis:alpine
```

### Redis with Password Authentication

```bash
podman run --rm -it -p 6379:6379 redis:alpine redis-server --requirepass "123456"
```

### Redis with TLS (Recommended for Production)

1. **Generate TLS Certificates** (already included in this directory):
   - `redis-cert.pem` - Server certificate
   - `redis-key.pem` - Private key

2. **Start Redis with TLS**:
   ```bash
   ./examples/redis/start-redis-tls.sh
   ```

   This starts Redis on port 6380 with TLS enabled and password `tls123`.

## Connection Examples

### Connect without Password

```php
use Async\Kernel;

Kernel::run(function () {
    $redis = new Redis();
    $redis->connect('redis://localhost:6379');

    $redis->set('key', 'value');
    $value = $redis->get('key');

    $redis->close();
});
```

### Connect with Password

```php
use Async\Kernel;

Kernel::run(function () {
    $redis = new Redis();
    $redis->connect('redis://:123456@localhost:6379');

    // Or connect first, then authenticate
    // $redis->connect('redis://localhost:6379');
    // $redis->auth('123456');

    $redis->close();
});
```

### Connect with TLS

```php
use Async\Kernel;

Kernel::run(function () {
    $redis = new Redis();

    // Use rediss:// (note the double 's')
    // #insecure flag skips certificate verification (for self-signed certs)
    $redis->connect('rediss://:tls123@localhost:6380#insecure');

    $redis->set('secure:key', 'encrypted value');
    $value = $redis->get('secure:key');

    $redis->close();
});
```

## Supported Redis Commands

### String Operations
- `set($key, $value, $timeout = null)` - Set key to value
- `get($key)` - Get value by key
- `setex($key, $ttl, $value)` - Set with expiration
- `incr($key)` - Increment value
- `incrBy($key, $value)` - Increment by amount
- `decr($key)` - Decrement value
- `decrBy($key, $value)` - Decrement by amount

### List Operations
- `lPush($key, ...$values)` - Push to head of list
- `rPush($key, ...$values)` - Push to tail of list
- `lPop($key)` - Pop from head
- `rPop($key)` - Pop from tail
- `lLen($key)` - Get list length
- `lRange($key, $start, $end)` - Get range of elements

### Hash Operations
- `hSet($key, $field, $value)` - Set hash field
- `hGet($key, $field)` - Get hash field
- `hGetAll($key)` - Get all fields
- `hDel($key, ...$fields)` - Delete hash fields
- `hExists($key, $field)` - Check if field exists

### Set Operations
- `sAdd($key, ...$members)` - Add members to set
- `sMembers($key)` - Get all set members
- `sIsMember($key, $member)` - Check membership
- `sRem($key, ...$members)` - Remove members from set

### Key Management
- `exists($keys)` - Check if key(s) exist
- `del($keys)` - Delete key(s)
- `expire($key, $seconds)` - Set expiration
- `ttl($key)` - Get time to live
- `ping($message = null)` - Ping server

## TLS Connection Tips

- **Scheme**: Use `rediss://` (with double 's') for TLS connections
- **Port**: Default TLS port is usually 6380 (vs 6379 for non-TLS)
- **Testing**: Append `#insecure` to skip certificate verification (self-signed certs only!)
- **Production**: Use valid certificates and proper verification
- **Authentication**: Include password in URL: `rediss://[user]:[password]@host:port`
- **Security**: TLS provides encryption for data in transit

## Files in This Directory

- **basic.php** - Basic Redis operations example
- **tls_connection.php** - TLS connection example with comprehensive operations
- **start-redis-tls.sh** - Script to start Redis with TLS using podman
- **redis-tls.conf** - Redis configuration for TLS mode
- **redis-cert.pem** - Self-signed TLS certificate (for testing)
- **redis-key.pem** - Private key for TLS certificate
- **README.md** - This file

## Troubleshooting

### Connection Refused

```
Error: Connection refused
```

**Solution**: Make sure Redis server is running on the specified port.

### Authentication Failed

```
Error: NOAUTH Authentication required
```

**Solution**: Provide the correct password in the connection URL or use `auth()` method.

### TLS Certificate Verification Failed

```
Error: certificate verify failed
```

**Solution**:
- For testing with self-signed certs, use `#insecure` flag
- For production, use valid certificates or configure proper CA bundle

### Extension Not Found

```
Fatal error: Class 'Redis' not found
```

**Solution**: Run the script from the project root directory with the extension loaded:
```bash
php -d extension=target/release/libasync_php.dylib examples/redis/basic.php
```

### WRONGPASS Invalid Password

```
Error: WRONGPASS invalid username-password pair
```

**Solution**: Check that the password in your connection URL matches the Redis server password.

## Notes

- All Redis operations must be wrapped in `Kernel::run()` for async execution
- The hooked `Redis` class automatically handles `Fiber::suspend()` internally
- Connection URLs support various formats: `redis://`, `rediss://` (TLS)
- For production use, always use TLS and strong passwords
- The `#insecure` flag should only be used for testing with self-signed certificates
