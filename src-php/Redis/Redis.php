<?php

namespace Redis;

use Async\Kernel\Redis\Client as KernelClient;
use Fiber;

/**
 * PHP Redis client wrapper
 *
 * This class wraps the Rust Kernel\Redis\Client and provides the full
 * PHP Redis extension API compatibility. It delegates actual Redis operations
 * to the Rust layer using async I/O.
 */
class Redis
{
    private KernelClient $kernel;

    public function __construct()
    {
        $this->kernel = new KernelClient();
    }

    /**
     * Connect to Redis server
     *
     * @param string $host Redis server host (default: "127.0.0.1")
     * @param int $port Redis server port (default: 6379)
     * @param float $timeout Connection timeout in seconds (default: 0.0, no timeout)
     * @param mixed $reserved Reserved parameter (unused, for compatibility)
     * @param int $retry_interval Retry interval in milliseconds (unused, for compatibility)
     * @param float $read_timeout Read timeout in seconds (default: 0.0, no timeout)
     * @return bool True on success, false on failure
     */
    public function connect(
        string $host = '127.0.0.1',
        int $port = 6379,
        float $timeout = 0.0,
        mixed $reserved = null,
        int $retry_interval = 0,
        float $read_timeout = 0.0
    ): bool {
        try {
            $result = Fiber::suspend($this->kernel->connect(
                $host,
                $port,
                $timeout,
                $reserved,
                $retry_interval,
                $read_timeout
            ));
            return (bool)$result;
        } catch (\Throwable $e) {
            return false;
        }
    }

    /**
     * Persistent connect to Redis server
     * Note: In async context, this behaves the same as connect()
     *
     * @param string $host Redis server host
     * @param int $port Redis server port
     * @param float $timeout Connection timeout in seconds
     * @param string|null $persistent_id Persistent connection ID (unused)
     * @param int $retry_interval Retry interval in milliseconds
     * @param float $read_timeout Read timeout in seconds
     * @return bool True on success, false on failure
     */
    public function pconnect(
        string $host = '127.0.0.1',
        int $port = 6379,
        float $timeout = 0.0,
        ?string $persistent_id = null,
        int $retry_interval = 0,
        float $read_timeout = 0.0
    ): bool {
        // In async context, persistent connections work the same as regular connections
        return $this->connect($host, $port, $timeout, $persistent_id, $retry_interval, $read_timeout);
    }

    /**
     * Close the Redis connection
     *
     * @return bool Always returns true
     */
    public function close(): bool
    {
        return $this->kernel->close();
    }

    /**
     * Ping the server
     *
     * @param string|null $message Optional message to send
     * @return string|bool Server response or false on failure
     */
    public function ping(?string $message = null): string|bool
    {
        try {
            return Fiber::suspend($this->kernel->ping($message));
        } catch (\Throwable $e) {
            return false;
        }
    }

    /**
     * Set string value
     *
     * @param string $key Key name
     * @param mixed $value Value to store
     * @param mixed $timeout Optional expiration in seconds or options array
     * @return bool True on success, false on failure
     */
    public function set(string $key, mixed $value, mixed $timeout = null): bool
    {
        try {
            $timeoutSeconds = null;

            // Handle different timeout formats
            if (is_int($timeout)) {
                $timeoutSeconds = $timeout;
            } elseif (is_array($timeout)) {
                // Handle ['EX' => seconds] or ['PX' => milliseconds] format
                if (isset($timeout['EX'])) {
                    $timeoutSeconds = (int)$timeout['EX'];
                } elseif (isset($timeout['PX'])) {
                    $timeoutSeconds = (int)($timeout['PX'] / 1000);
                }
            }

            $result = Fiber::suspend($this->kernel->set($key, (string)$value, $timeoutSeconds));
            return $result !== false;
        } catch (\Throwable $e) {
            return false;
        }
    }

    /**
     * Set string value with expiration
     *
     * @param string $key Key name
     * @param int $ttl Time to live in seconds
     * @param mixed $value Value to store
     * @return bool True on success, false on failure
     */
    public function setex(string $key, int $ttl, mixed $value): bool
    {
        return $this->set($key, $value, $ttl);
    }

    /**
     * Get string value
     *
     * @param string $key Key name
     * @return mixed Value or false if key doesn't exist
     */
    public function get(string $key): mixed
    {
        try {
            return Fiber::suspend($this->kernel->get($key));
        } catch (\Throwable $e) {
            return false;
        }
    }

    /**
     * Delete one or more keys
     *
     * @param string|array $key Key name or array of keys
     * @param string ...$otherKeys Additional keys to delete
     * @return int|false Number of keys deleted or false on failure
     */
    public function del(string|array $key, string ...$otherKeys): int|false
    {
        try {
            // Combine all keys into an array
            $keys = is_array($key) ? $key : [$key, ...$otherKeys];
            return (int)Fiber::suspend($this->kernel->del($keys));
        } catch (\Throwable $e) {
            return false;
        }
    }

    /**
     * Alias for del()
     */
    public function delete(string|array $key, string ...$otherKeys): int|false
    {
        return $this->del($key, ...$otherKeys);
    }

    /**
     * Check if key exists
     *
     * @param string|array $key Key name or array of keys
     * @return int|false Number of existing keys or false on failure
     */
    public function exists(string|array $key): int|false
    {
        try {
            if (is_array($key)) {
                // For multiple keys, check each one
                $count = 0;
                foreach ($key as $k) {
                    $count += (int)Fiber::suspend($this->kernel->exists($k));
                }
                return $count;
            }
            return (int)Fiber::suspend($this->kernel->exists($key));
        } catch (\Throwable $e) {
            return false;
        }
    }

    /**
     * Set expiration on key
     *
     * @param string $key Key name
     * @param int $seconds Expiration in seconds
     * @return bool True if timeout was set, false otherwise
     */
    public function expire(string $key, int $seconds): bool
    {
        try {
            return (bool)Fiber::suspend($this->kernel->expire($key, $seconds));
        } catch (\Throwable $e) {
            return false;
        }
    }

    /**
     * Get time to live for key
     *
     * @param string $key Key name
     * @return int|false TTL in seconds, -1 if no expiry, -2 if key doesn't exist, false on failure
     */
    public function ttl(string $key): int|false
    {
        try {
            return (int)Fiber::suspend($this->kernel->ttl($key));
        } catch (\Throwable $e) {
            return false;
        }
    }

    /**
     * Increment value
     *
     * @param string $key Key name
     * @return int|false New value or false on failure
     */
    public function incr(string $key): int|false
    {
        try {
            return (int)Fiber::suspend($this->kernel->incr($key));
        } catch (\Throwable $e) {
            return false;
        }
    }

    /**
     * Increment by value
     *
     * @param string $key Key name
     * @param int $value Amount to increment by
     * @return int|false New value or false on failure
     */
    public function incrBy(string $key, int $value): int|false
    {
        try {
            return (int)Fiber::suspend($this->kernel->incrBy($key, $value));
        } catch (\Throwable $e) {
            return false;
        }
    }

    /**
     * Decrement value
     *
     * @param string $key Key name
     * @return int|false New value or false on failure
     */
    public function decr(string $key): int|false
    {
        try {
            return (int)Fiber::suspend($this->kernel->decr($key));
        } catch (\Throwable $e) {
            return false;
        }
    }

    /**
     * Decrement by value
     *
     * @param string $key Key name
     * @param int $value Amount to decrement by
     * @return int|false New value or false on failure
     */
    public function decrBy(string $key, int $value): int|false
    {
        try {
            return (int)Fiber::suspend($this->kernel->decrBy($key, $value));
        } catch (\Throwable $e) {
            return false;
        }
    }

    /**
     * Push value(s) to list (left)
     *
     * @param string $key List key name
     * @param mixed ...$values Values to push
     * @return int|false New list length or false on failure
     */
    public function lPush(string $key, mixed ...$values): int|false
    {
        try {
            $stringValues = array_map('strval', $values);
            return (int)Fiber::suspend($this->kernel->lPush($key, $stringValues));
        } catch (\Throwable $e) {
            return false;
        }
    }

    /**
     * Push value(s) to list (right)
     *
     * @param string $key List key name
     * @param mixed ...$values Values to push
     * @return int|false New list length or false on failure
     */
    public function rPush(string $key, mixed ...$values): int|false
    {
        try {
            $stringValues = array_map('strval', $values);
            return (int)Fiber::suspend($this->kernel->rPush($key, $stringValues));
        } catch (\Throwable $e) {
            return false;
        }
    }

    /**
     * Pop value from list (left)
     *
     * @param string $key List key name
     * @return mixed Value or false if list is empty
     */
    public function lPop(string $key): mixed
    {
        try {
            return Fiber::suspend($this->kernel->lPop($key));
        } catch (\Throwable $e) {
            return false;
        }
    }

    /**
     * Pop value from list (right)
     *
     * @param string $key List key name
     * @return mixed Value or false if list is empty
     */
    public function rPop(string $key): mixed
    {
        try {
            return Fiber::suspend($this->kernel->rPop($key));
        } catch (\Throwable $e) {
            return false;
        }
    }

    /**
     * Get list length
     *
     * @param string $key List key name
     * @return int|false List length or false on failure
     */
    public function lLen(string $key): int|false
    {
        try {
            return (int)Fiber::suspend($this->kernel->lLen($key));
        } catch (\Throwable $e) {
            return false;
        }
    }

    /**
     * Alias for lLen()
     */
    public function lSize(string $key): int|false
    {
        return $this->lLen($key);
    }

    /**
     * Get list range
     *
     * @param string $key List key name
     * @param int $start Start index
     * @param int $stop Stop index
     * @return array|false Array of values or false on failure
     */
    public function lRange(string $key, int $start, int $stop): array|false
    {
        try {
            $result = Fiber::suspend($this->kernel->lRange($key, $start, $stop));
            return is_array($result) ? $result : false;
        } catch (\Throwable $e) {
            return false;
        }
    }

    /**
     * Alias for lRange()
     */
    public function lGetRange(string $key, int $start, int $stop): array|false
    {
        return $this->lRange($key, $start, $stop);
    }

    /**
     * Set hash field
     *
     * @param string $key Hash key name
     * @param string $field Field name
     * @param mixed $value Field value
     * @return int|false 1 if new field, 0 if updated, false on failure
     */
    public function hSet(string $key, string $field, mixed $value): int|false
    {
        try {
            return (int)Fiber::suspend($this->kernel->hSet($key, $field, (string)$value));
        } catch (\Throwable $e) {
            return false;
        }
    }

    /**
     * Get hash field
     *
     * @param string $key Hash key name
     * @param string $field Field name
     * @return mixed Field value or false if field doesn't exist
     */
    public function hGet(string $key, string $field): mixed
    {
        try {
            return Fiber::suspend($this->kernel->hGet($key, $field));
        } catch (\Throwable $e) {
            return false;
        }
    }

    /**
     * Get all hash fields and values
     *
     * @param string $key Hash key name
     * @return array|false Associative array of fields and values or false on failure
     */
    public function hGetAll(string $key): array|false
    {
        try {
            $result = Fiber::suspend($this->kernel->hGetAll($key));
            if (!is_array($result)) {
                return false;
            }

            // Convert flat array [k1, v1, k2, v2, ...] to associative array
            $hash = [];
            for ($i = 0; $i < count($result); $i += 2) {
                if (isset($result[$i + 1])) {
                    $hash[$result[$i]] = $result[$i + 1];
                }
            }
            return $hash;
        } catch (\Throwable $e) {
            return false;
        }
    }

    /**
     * Delete hash field(s)
     *
     * @param string $key Hash key name
     * @param string ...$fields Field names to delete
     * @return int|false Number of fields deleted or false on failure
     */
    public function hDel(string $key, string ...$fields): int|false
    {
        try {
            return (int)Fiber::suspend($this->kernel->hDel($key, $fields));
        } catch (\Throwable $e) {
            return false;
        }
    }

    /**
     * Check if hash field exists
     *
     * @param string $key Hash key name
     * @param string $field Field name
     * @return bool True if field exists, false otherwise
     */
    public function hExists(string $key, string $field): bool
    {
        try {
            return (bool)Fiber::suspend($this->kernel->hExists($key, $field));
        } catch (\Throwable $e) {
            return false;
        }
    }

    /**
     * Add member(s) to set
     *
     * @param string $key Set key name
     * @param mixed ...$values Members to add
     * @return int|false Number of members added or false on failure
     */
    public function sAdd(string $key, mixed ...$values): int|false
    {
        try {
            $stringValues = array_map('strval', $values);
            return (int)Fiber::suspend($this->kernel->sAdd($key, $stringValues));
        } catch (\Throwable $e) {
            return false;
        }
    }

    /**
     * Get all set members
     *
     * @param string $key Set key name
     * @return array|false Array of members or false on failure
     */
    public function sMembers(string $key): array|false
    {
        try {
            $result = Fiber::suspend($this->kernel->sMembers($key));
            return is_array($result) ? $result : false;
        } catch (\Throwable $e) {
            return false;
        }
    }

    /**
     * Alias for sMembers()
     */
    public function sGetMembers(string $key): array|false
    {
        return $this->sMembers($key);
    }

    /**
     * Check if member exists in set
     *
     * @param string $key Set key name
     * @param mixed $value Member to check
     * @return bool True if member exists, false otherwise
     */
    public function sIsMember(string $key, mixed $value): bool
    {
        try {
            return (bool)Fiber::suspend($this->kernel->sIsMember($key, (string)$value));
        } catch (\Throwable $e) {
            return false;
        }
    }

    /**
     * Alias for sIsMember()
     */
    public function sContains(string $key, mixed $value): bool
    {
        return $this->sIsMember($key, $value);
    }

    /**
     * Remove member(s) from set
     *
     * @param string $key Set key name
     * @param mixed ...$values Members to remove
     * @return int|false Number of members removed or false on failure
     */
    public function sRem(string $key, mixed ...$values): int|false
    {
        try {
            $stringValues = array_map('strval', $values);
            return (int)Fiber::suspend($this->kernel->sRem($key, $stringValues));
        } catch (\Throwable $e) {
            return false;
        }
    }

    /**
     * Alias for sRem()
     */
    public function sRemove(string $key, mixed ...$values): int|false
    {
        return $this->sRem($key, ...$values);
    }

    /**
     * Get last error message
     *
     * @return string|null Error message or null
     */
    public function getLastError(): ?string
    {
        $error = $this->kernel->getLastError();
        return empty($error) ? null : $error;
    }

    /**
     * Clear last error
     */
    public function clearLastError(): void
    {
        $this->kernel->clearLastError();
    }

    public function publish(string $channel, string $message): int|false
    {
        try {
            return (int)Fiber::suspend($this->kernel->publish($channel, $message));
        } catch (\Throwable $e) {
            return false;
        }
    }

    public function subscribe(array $channels, callable $callback): bool
    {
        try {
            Fiber::suspend($this->kernel->subscribe($channels, $callback));
            return true;
        } catch (\Throwable $e) {
            return false;
        }
    }
}
