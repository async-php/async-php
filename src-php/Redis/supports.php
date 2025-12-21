<?php
/**
 * Global Redis functions for async-php
 *
 * These functions provide convenience wrappers around the Redis class,
 * making it easier to work with Redis in a procedural style.
 * They hook the native PHP Redis extension functions.
 */

if (!function_exists('redis_connect')) {
    function redis_connect(string $host = '127.0.0.1', int $port = 6379, float $timeout = 0.0): Redis|false {
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
}

if (!function_exists('redis_pconnect')) {
    function redis_pconnect(string $host = '127.0.0.1', int $port = 6379, float $timeout = 0.0, ?string $persistent_id = null): Redis|false {
        return redis_connect($host, $port, $timeout);
    }
}

if (!function_exists('redis_ping')) {
    function redis_ping(Redis $redis, ?string $message = null): string|bool {
        return $redis->ping($message);
    }
}

if (!function_exists('redis_set')) {
    function redis_set(Redis $redis, string $key, mixed $value, mixed $timeout = null): bool {
        return $redis->set($key, $value, $timeout);
    }
}

if (!function_exists('redis_get')) {
    function redis_get(Redis $redis, string $key): mixed {
        return $redis->get($key);
    }
}

if (!function_exists('redis_del')) {
    function redis_del(Redis $redis, string|array $key, string ...$otherKeys): int|false {
        return $redis->del($key, ...$otherKeys);
    }
}

if (!function_exists('redis_exists')) {
    function redis_exists(Redis $redis, string|array $key): int|false {
        return $redis->exists($key);
    }
}

if (!function_exists('redis_expire')) {
    function redis_expire(Redis $redis, string $key, int $seconds): bool {
        return $redis->expire($key, $seconds);
    }
}

if (!function_exists('redis_ttl')) {
    function redis_ttl(Redis $redis, string $key): int|false {
        return $redis->ttl($key);
    }
}

if (!function_exists('redis_incr')) {
    function redis_incr(Redis $redis, string $key): int|false {
        return $redis->incr($key);
    }
}

if (!function_exists('redis_incrby')) {
    function redis_incrby(Redis $redis, string $key, int $value): int|false {
        return $redis->incrBy($key, $value);
    }
}

if (!function_exists('redis_decr')) {
    function redis_decr(Redis $redis, string $key): int|false {
        return $redis->decr($key);
    }
}

if (!function_exists('redis_decrby')) {
    function redis_decrby(Redis $redis, string $key, int $value): int|false {
        return $redis->decrBy($key, $value);
    }
}

if (!function_exists('redis_lpush')) {
    function redis_lpush(Redis $redis, string $key, mixed ...$values): int|false {
        return $redis->lPush($key, ...$values);
    }
}

if (!function_exists('redis_rpush')) {
    function redis_rpush(Redis $redis, string $key, mixed ...$values): int|false {
        return $redis->rPush($key, ...$values);
    }
}

if (!function_exists('redis_lpop')) {
    function redis_lpop(Redis $redis, string $key): mixed {
        return $redis->lPop($key);
    }
}

if (!function_exists('redis_rpop')) {
    function redis_rpop(Redis $redis, string $key): mixed {
        return $redis->rPop($key);
    }
}

if (!function_exists('redis_llen')) {
    function redis_llen(Redis $redis, string $key): int|false {
        return $redis->lLen($key);
    }
}

if (!function_exists('redis_lrange')) {
    function redis_lrange(Redis $redis, string $key, int $start, int $stop): array|false {
        return $redis->lRange($key, $start, $stop);
    }
}

if (!function_exists('redis_hset')) {
    function redis_hset(Redis $redis, string $key, string $field, mixed $value): int|false {
        return $redis->hSet($key, $field, $value);
    }
}

if (!function_exists('redis_hget')) {
    function redis_hget(Redis $redis, string $key, string $field): mixed {
        return $redis->hGet($key, $field);
    }
}

if (!function_exists('redis_hgetall')) {
    function redis_hgetall(Redis $redis, string $key): array|false {
        return $redis->hGetAll($key);
    }
}

if (!function_exists('redis_hdel')) {
    function redis_hdel(Redis $redis, string $key, string ...$fields): int|false {
        return $redis->hDel($key, ...$fields);
    }
}

if (!function_exists('redis_hexists')) {
    function redis_hexists(Redis $redis, string $key, string $field): bool {
        return $redis->hExists($key, $field);
    }
}

if (!function_exists('redis_sadd')) {
    function redis_sadd(Redis $redis, string $key, mixed ...$values): int|false {
        return $redis->sAdd($key, ...$values);
    }
}

if (!function_exists('redis_smembers')) {
    function redis_smembers(Redis $redis, string $key): array|false {
        return $redis->sMembers($key);
    }
}

if (!function_exists('redis_sismember')) {
    function redis_sismember(Redis $redis, string $key, mixed $value): bool {
        return $redis->sIsMember($key, $value);
    }
}

if (!function_exists('redis_srem')) {
    function redis_srem(Redis $redis, string $key, mixed ...$values): int|false {
        return $redis->sRem($key, ...$values);
    }
}

if (!function_exists('redis_close')) {
    function redis_close(Redis $redis): bool {
        return $redis->close();
    }
}

// Hook native Redis class with class_alias
if (!class_exists('Redis', false)) {
    class_alias(\Redis\Redis::class, 'Redis');
}
