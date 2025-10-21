<?php

namespace Async\Database;

use AsyncMySql;
use Fiber;

class MySQL implements DriverInterface
{
    private AsyncMySql $driver;

    private function __construct(AsyncMySql $driver)
    {
        $this->driver = $driver;
    }

    public static function connect(string $dsn, int $maxConnections = 10): ?static
    {
        // DSN Example: mysql://user:pass@127.0.0.1:3306/db
        $driver = Fiber::suspend(AsyncMySql::connect($dsn, $maxConnections));
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
