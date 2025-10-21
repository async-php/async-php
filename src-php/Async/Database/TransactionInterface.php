<?php

namespace Async\Database;

interface TransactionInterface
{
    public function query(string $sql, ?array $params = null): array|false;
    public function execute(string $sql, ?array $params = null): int|false;
    public function commit(): bool;
    public function rollback(): bool;
}
