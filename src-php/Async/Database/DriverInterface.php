<?php

namespace Async\Database;

interface DriverInterface
{
    public static function connect(string $dsn, int $maxConnections = 10): ?static;

    /**
     * @param string $sql
     * @param array|null $params
     * @return array|false
     */
    public function query(string $sql, ?array $params = null): array|false;

    /**
     * @param string $sql
     * @param array|null $params
     * @return int|false
     */
    public function execute(string $sql, ?array $params = null): int|false;

    /**
     * Begin a transaction. Returns a Transaction object.
     *
     * @return TransactionInterface|false
     */
    public function beginTransaction(): TransactionInterface|false;
}