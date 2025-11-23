<?php

namespace Pdo;

/**
 * MySQL specialized PDO class (PHP 8.4+ naming).
 *
 * This class intentionally exists even when native PDO is disabled, to keep
 * user code aligned with the latest PHP driver class names.
 */
class Mysql extends \PDO
{
    public function __construct(string $dsn, ?string $username = null, ?string $password = null, ?array $options = null)
    {
        $dsn = self::normalizeDsn($dsn);
        parent::__construct($dsn, $username, $password, $options);

        $options ??= [];
        if (array_key_exists(\PDO::MYSQL_ATTR_INIT_COMMAND, $options)) {
            $cmd = (string)$options[\PDO::MYSQL_ATTR_INIT_COMMAND];
            if ($cmd !== '') {
                $this->exec($cmd);
            }
        }

        $charset = self::extractDsnValue($dsn, 'charset');
        if ($charset !== null) {
            $this->setCharset($charset);
        }
    }

    public function getServerVersion(): string|false
    {
        $stmt = $this->query('SELECT VERSION() AS v');
        if ($stmt === false) {
            return false;
        }
        $v = $stmt->fetchColumn(0);
        return $v === false ? false : (string)$v;
    }

    public function getConnectionId(): int|false
    {
        $stmt = $this->query('SELECT CONNECTION_ID() AS id');
        if ($stmt === false) {
            return false;
        }
        $v = $stmt->fetchColumn(0);
        return $v === false ? false : (int)$v;
    }

    public function setCharset(string $charset): bool
    {
        if (preg_match('/^[A-Za-z0-9_\\-]+$/', $charset) !== 1) {
            return false;
        }
        return $this->exec('SET NAMES ' . $charset) !== false;
    }

    private static function normalizeDsn(string $dsn): string
    {
        $dsn = ltrim($dsn);
        if (str_starts_with($dsn, 'mysql:')) {
            return $dsn;
        }
        if (str_starts_with($dsn, 'mysql://')) {
            return $dsn;
        }
        return 'mysql:' . $dsn;
    }

    private static function extractDsnValue(string $dsn, string $key): ?string
    {
        if (!str_starts_with($dsn, 'mysql:')) {
            return null;
        }
        $rest = substr($dsn, 6);
        foreach (explode(';', $rest) as $part) {
            $part = trim($part);
            if ($part === '') {
                continue;
            }
            $eq = strpos($part, '=');
            if ($eq === false) {
                continue;
            }
            $k = strtolower(trim(substr($part, 0, $eq)));
            if ($k !== strtolower($key)) {
                continue;
            }
            return trim(substr($part, $eq + 1));
        }
        return null;
    }
}

