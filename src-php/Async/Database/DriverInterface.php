<?php

namespace Async\Database;

interface DriverInterface
{
    /**
     * Connect to database.
     *
     * @param string $dsn
     * @param int $maxConnections
     * @return static
     */
    public static function connect(string $dsn, int $maxConnections = 10): ?static;

    /**
     * Execute a query and return result set (array of arrays).
     *
     * @param string $sql
     * @return array|false
     */
    public function query(string $sql): array|false;

    /**
     * Execute a statement and return affected rows.
     *
     * @param string $sql
     * @return int|false
     */
    public function execute(string $sql): int|false;
}
