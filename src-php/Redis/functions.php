<?php

namespace Redis;

/**
 * Global Redis functions for async-php
 *
 * These functions provide convenience wrappers around the Redis class,
 * making it easier to work with Redis in a procedural style.
 */

/**
 * Create a new Redis connection
 *
 * @param string $host Redis server host (default: "127.0.0.1")
 * @param int $port Redis server port (default: 6379)
 * @param float $timeout Connection timeout in seconds (default: 0.0)
 * @return Redis|false Redis instance or false on failure
 */
function redis_connect(
    string $host = '127.0.0.1',
    int $port = 6379,
    float $timeout = 0.0
): Redis|false {
    try {
        $redis = new Redis();
        if ($redis->connect($host, $port, $timeout)) {
            return $redis;
        }
        return false;
    } catch (\Throwable $e) {
        return false;
    }
}

/**
 * Create a persistent Redis connection
 * Note: In async context, this behaves the same as redis_connect()
 *
 * @param string $host Redis server host
 * @param int $port Redis server port
 * @param float $timeout Connection timeout in seconds
 * @param string|null $persistent_id Persistent connection ID (unused)
 * @return Redis|false Redis instance or false on failure
 */
function redis_pconnect(
    string $host = '127.0.0.1',
    int $port = 6379,
    float $timeout = 0.0,
    ?string $persistent_id = null
): Redis|false {
    return redis_connect($host, $port, $timeout);
}

/**
 * Ping the Redis server
 *
 * @param Redis $redis Redis instance
 * @param string|null $message Optional message to send
 * @return string|bool Server response or false on failure
 */
function redis_ping(Redis $redis, ?string $message = null): string|bool
{
    return $redis->ping($message);
}

/**
 * Set a string value
 *
 * @param Redis $redis Redis instance
 * @param string $key Key name
 * @param mixed $value Value to store
 * @param mixed $timeout Optional expiration in seconds
 * @return bool True on success, false on failure
 */
function redis_set(Redis $redis, string $key, mixed $value, mixed $timeout = null): bool
{
    return $redis->set($key, $value, $timeout);
}

/**
 * Get a string value
 *
 * @param Redis $redis Redis instance
 * @param string $key Key name
 * @return mixed Value or false if key doesn't exist
 */
function redis_get(Redis $redis, string $key): mixed
{
    return $redis->get($key);
}

/**
 * Delete one or more keys
 *
 * @param Redis $redis Redis instance
 * @param string|array $key Key name or array of keys
 * @param string ...$otherKeys Additional keys to delete
 * @return int|false Number of keys deleted or false on failure
 */
function redis_del(Redis $redis, string|array $key, string ...$otherKeys): int|false
{
    return $redis->del($key, ...$otherKeys);
}

/**
 * Check if key exists
 *
 * @param Redis $redis Redis instance
 * @param string|array $key Key name or array of keys
 * @return int|false Number of existing keys or false on failure
 */
function redis_exists(Redis $redis, string|array $key): int|false
{
    return $redis->exists($key);
}

/**
 * Set expiration on key
 *
 * @param Redis $redis Redis instance
 * @param string $key Key name
 * @param int $seconds Expiration in seconds
 * @return bool True if timeout was set, false otherwise
 */
function redis_expire(Redis $redis, string $key, int $seconds): bool
{
    return $redis->expire($key, $seconds);
}

/**
 * Get time to live for key
 *
 * @param Redis $redis Redis instance
 * @param string $key Key name
 * @return int|false TTL in seconds or false on failure
 */
function redis_ttl(Redis $redis, string $key): int|false
{
    return $redis->ttl($key);
}

/**
 * Increment value
 *
 * @param Redis $redis Redis instance
 * @param string $key Key name
 * @return int|false New value or false on failure
 */
function redis_incr(Redis $redis, string $key): int|false
{
    return $redis->incr($key);
}

/**
 * Increment by value
 *
 * @param Redis $redis Redis instance
 * @param string $key Key name
 * @param int $value Amount to increment by
 * @return int|false New value or false on failure
 */
function redis_incrby(Redis $redis, string $key, int $value): int|false
{
    return $redis->incrBy($key, $value);
}

/**
 * Decrement value
 *
 * @param Redis $redis Redis instance
 * @param string $key Key name
 * @return int|false New value or false on failure
 */
function redis_decr(Redis $redis, string $key): int|false
{
    return $redis->decr($key);
}

/**
 * Decrement by value
 *
 * @param Redis $redis Redis instance
 * @param string $key Key name
 * @param int $value Amount to decrement by
 * @return int|false New value or false on failure
 */
function redis_decrby(Redis $redis, string $key, int $value): int|false
{
    return $redis->decrBy($key, $value);
}

/**
 * Push value(s) to list (left)
 *
 * @param Redis $redis Redis instance
 * @param string $key List key name
 * @param mixed ...$values Values to push
 * @return int|false New list length or false on failure
 */
function redis_lpush(Redis $redis, string $key, mixed ...$values): int|false
{
    return $redis->lPush($key, ...$values);
}

/**
 * Push value(s) to list (right)
 *
 * @param Redis $redis Redis instance
 * @param string $key List key name
 * @param mixed ...$values Values to push
 * @return int|false New list length or false on failure
 */
function redis_rpush(Redis $redis, string $key, mixed ...$values): int|false
{
    return $redis->rPush($key, ...$values);
}

/**
 * Pop value from list (left)
 *
 * @param Redis $redis Redis instance
 * @param string $key List key name
 * @return mixed Value or false if list is empty
 */
function redis_lpop(Redis $redis, string $key): mixed
{
    return $redis->lPop($key);
}

/**
 * Pop value from list (right)
 *
 * @param Redis $redis Redis instance
 * @param string $key List key name
 * @return mixed Value or false if list is empty
 */
function redis_rpop(Redis $redis, string $key): mixed
{
    return $redis->rPop($key);
}

/**
 * Get list length
 *
 * @param Redis $redis Redis instance
 * @param string $key List key name
 * @return int|false List length or false on failure
 */
function redis_llen(Redis $redis, string $key): int|false
{
    return $redis->lLen($key);
}

/**
 * Get list range
 *
 * @param Redis $redis Redis instance
 * @param string $key List key name
 * @param int $start Start index
 * @param int $stop Stop index
 * @return array|false Array of values or false on failure
 */
function redis_lrange(Redis $redis, string $key, int $start, int $stop): array|false
{
    return $redis->lRange($key, $start, $stop);
}

/**
 * Set hash field
 *
 * @param Redis $redis Redis instance
 * @param string $key Hash key name
 * @param string $field Field name
 * @param mixed $value Field value
 * @return int|false 1 if new field, 0 if updated, false on failure
 */
function redis_hset(Redis $redis, string $key, string $field, mixed $value): int|false
{
    return $redis->hSet($key, $field, $value);
}

/**
 * Get hash field
 *
 * @param Redis $redis Redis instance
 * @param string $key Hash key name
 * @param string $field Field name
 * @return mixed Field value or false if field doesn't exist
 */
function redis_hget(Redis $redis, string $key, string $field): mixed
{
    return $redis->hGet($key, $field);
}

/**
 * Get all hash fields and values
 *
 * @param Redis $redis Redis instance
 * @param string $key Hash key name
 * @return array|false Associative array or false on failure
 */
function redis_hgetall(Redis $redis, string $key): array|false
{
    return $redis->hGetAll($key);
}

/**
 * Delete hash field(s)
 *
 * @param Redis $redis Redis instance
 * @param string $key Hash key name
 * @param string ...$fields Field names to delete
 * @return int|false Number of fields deleted or false on failure
 */
function redis_hdel(Redis $redis, string $key, string ...$fields): int|false
{
    return $redis->hDel($key, ...$fields);
}

/**
 * Check if hash field exists
 *
 * @param Redis $redis Redis instance
 * @param string $key Hash key name
 * @param string $field Field name
 * @return bool True if field exists, false otherwise
 */
function redis_hexists(Redis $redis, string $key, string $field): bool
{
    return $redis->hExists($key, $field);
}

/**
 * Add member(s) to set
 *
 * @param Redis $redis Redis instance
 * @param string $key Set key name
 * @param mixed ...$values Members to add
 * @return int|false Number of members added or false on failure
 */
function redis_sadd(Redis $redis, string $key, mixed ...$values): int|false
{
    return $redis->sAdd($key, ...$values);
}

/**
 * Get all set members
 *
 * @param Redis $redis Redis instance
 * @param string $key Set key name
 * @return array|false Array of members or false on failure
 */
function redis_smembers(Redis $redis, string $key): array|false
{
    return $redis->sMembers($key);
}

/**
 * Check if member exists in set
 *
 * @param Redis $redis Redis instance
 * @param string $key Set key name
 * @param mixed $value Member to check
 * @return bool True if member exists, false otherwise
 */
function redis_sismember(Redis $redis, string $key, mixed $value): bool
{
    return $redis->sIsMember($key, $value);
}

/**
 * Remove member(s) from set
 *
 * @param Redis $redis Redis instance
 * @param string $key Set key name
 * @param mixed ...$values Members to remove
 * @return int|false Number of members removed or false on failure
 */
function redis_srem(Redis $redis, string $key, mixed ...$values): int|false
{
    return $redis->sRem($key, ...$values);
}

/**
 * Close Redis connection
 *
 * @param Redis $redis Redis instance
 * @return bool Always returns true
 */
function redis_close(Redis $redis): bool
{
    return $redis->close();
}
