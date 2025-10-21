<?php

namespace Async\Database;

use AsyncPgSql;
use Fiber;

class PostgreSQL implements DriverInterface
{
    private AsyncPgSql $driver;

    private function __construct(AsyncPgSql $driver)
    {
        $this->driver = $driver;
    }

    public static function connect(string $dsn, int $maxConnections = 10): ?static
    {
        // DSN Example: postgres://user:pass@127.0.0.1:5432/db
        $driver = Fiber::suspend(AsyncPgSql::connect($dsn, $maxConnections));
        if (!$driver) return null;
        
        return new static($driver);
    }

    public function query(string $sql): array|false
    {
        return Fiber::suspend($this->driver->query($sql));
    }

    public function execute(string $sql): int|false
    {
        return Fiber::suspend($this->driver->execute($sql));
    }
}
