<?php

namespace Pdo;

/**
 * PostgreSQL specialized PDO class (PHP 8.4+ naming).
 */
class Pgsql extends \PDO
{
    public function __construct(string $dsn, ?string $username = null, ?string $password = null, ?array $options = null)
    {
        $dsn = self::normalizeDsn($dsn);
        parent::__construct($dsn, $username, $password, $options);

        $options ??= [];
        if (array_key_exists(\PDO::PGSQL_ATTR_DISABLE_PREPARES, $options)) {
            $this->setAttribute(\PDO::PGSQL_ATTR_DISABLE_PREPARES, (bool)$options[\PDO::PGSQL_ATTR_DISABLE_PREPARES]);
        }
    }

    public function getServerVersion(): string|false
    {
        $stmt = $this->query('SHOW server_version');
        if ($stmt === false) {
            return false;
        }
        $v = $stmt->fetchColumn(0);
        return $v === false ? false : (string)$v;
    }

    public function getBackendPid(): int|false
    {
        $stmt = $this->query('SELECT pg_backend_pid()');
        if ($stmt === false) {
            return false;
        }
        $v = $stmt->fetchColumn(0);
        return $v === false ? false : (int)$v;
    }

    public function notify(string $channel, ?string $payload = null): bool
    {
        $stmt = $this->prepare('SELECT pg_notify(?, ?)');
        if ($stmt === false) {
            return false;
        }
        return $stmt->execute([$channel, $payload ?? '']);
    }

    private static function normalizeDsn(string $dsn): string
    {
        $dsn = ltrim($dsn);
        if (str_starts_with($dsn, 'pgsql:')) {
            return $dsn;
        }
        if (str_starts_with($dsn, 'postgres:')) {
            return 'pgsql:' . substr($dsn, 8);
        }
        if (str_starts_with($dsn, 'postgres://') || str_starts_with($dsn, 'postgresql://')) {
            // Allow SQLx-style URI passthrough via PDO's constructor too.
            return 'pgsql:' . $dsn;
        }
        return 'pgsql:' . $dsn;
    }
}

