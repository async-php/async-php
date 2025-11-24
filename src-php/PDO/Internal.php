<?php

namespace PDO;

final class Internal
{
    /**
     * Parse a PDO-style DSN (mysql:/pgsql:) into a SQLx URI.
     *
     * @return array{0:"mysql"|"pgsql",1:string}
     */
    public static function dsnToSqlx(string $dsn, ?string $username, ?string $password): array
    {
        $pos = strpos($dsn, ':');
        if ($pos === false) {
            throw new \InvalidArgumentException("Invalid DSN (missing scheme): $dsn");
        }

        $scheme = strtolower(substr($dsn, 0, $pos));
        $rest = substr($dsn, $pos + 1);

        if ($scheme !== 'mysql' && $scheme !== 'pgsql') {
            throw new \InvalidArgumentException("Unsupported PDO driver: $scheme");
        }

        $pairs = self::parseDsnPairs($rest);
        if (isset($pairs['__uri']) && $pairs['__uri'] !== '') {
            $uri = (string)$pairs['__uri'];
            if ($scheme === 'pgsql') {
                $uri = preg_replace('/^pgsql:\\/\\//i', 'postgres://', $uri) ?? $uri;
            }
            return [$scheme, $uri];
        }

        $host = (string)($pairs['host'] ?? 'localhost');
        $port = $pairs['port'] ?? null;
        $dbname = (string)($pairs['dbname'] ?? '');

        $user = $username ?? (string)($pairs['user'] ?? '');
        $pass = $password ?? (string)($pairs['password'] ?? $pairs['pass'] ?? '');

        $query = [];
        foreach ($pairs as $k => $v) {
            $lk = strtolower($k);
            if (in_array($lk, ['host', 'port', 'dbname', 'user', 'username', 'password', 'pass'], true)) {
                continue;
            }
            if ($scheme === 'mysql' && $lk === 'unix_socket') {
                $lk = 'socket';
            }
            $query[$lk] = (string)$v;
        }

        $auth = '';
        if ($user !== '') {
            $auth = rawurlencode($user);
            if ($pass !== '') {
                $auth .= ':' . rawurlencode($pass);
            }
            $auth .= '@';
        }

        $hostPart = $host;
        if ($scheme === 'pgsql' && str_starts_with($hostPart, '/')) {
            // Use query param `host=/path` for unix socket mode.
            $query['host'] = $hostPart;
            $hostPart = 'localhost';
        }
        if ($port !== null && $port !== '') {
            $hostPart .= ':' . (int)$port;
        }

        $path = $dbname !== '' ? '/' . rawurlencode($dbname) : '';
        $qs = $query ? ('?' . http_build_query($query, '', '&', PHP_QUERY_RFC3986)) : '';

        $uriScheme = $scheme === 'pgsql' ? 'postgres' : 'mysql';
        return [$scheme, $uriScheme . '://' . $auth . $hostPart . $path . $qs];
    }

    /**
     * @return array<string,string>
     */
    private static function parseDsnPairs(string $rest): array
    {
        $rest = ltrim($rest);
        if ($rest === '') {
            return [];
        }

        // Allow URI-ish DSNs too (mysql://... or postgres://...), pass through.
        if (str_contains($rest, '://')) {
            return ['__uri' => $rest];
        }

        $out = [];
        foreach (explode(';', $rest) as $chunk) {
            $chunk = trim($chunk);
            if ($chunk === '') {
                continue;
            }
            $eq = strpos($chunk, '=');
            if ($eq === false) {
                $out[$chunk] = '';
                continue;
            }
            $k = trim(substr($chunk, 0, $eq));
            $v = trim(substr($chunk, $eq + 1));
            $out[$k] = $v;
        }
        return $out;
    }

    /**
     * Compile PDO-style placeholders into driver-native ones.
     * Uses high-performance Rust implementation.
     *
     * @return array{0:string,1:list<array{kind:"pos"|"named",key:int|string}>}
     */
    public static function compilePlaceholders(string $driver, string $sql): array
    {
        $result = sql_compile_placeholders($driver, $sql);
        return [$result['sql'], $result['placeholders']];
    }

    /**
     * @param int|string $param
     * @return int|string
     */
    public static function normalizeParamKey(int|string $param): int|string
    {
        if (is_string($param)) {
            return ltrim($param, ':');
        }
        return $param;
    }
}
