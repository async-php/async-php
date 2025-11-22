<?php

namespace PDO;

final class PDOStatement
{
    public string $queryString;

    private PDO $pdo;
    private string $sql;
    private string $compiledSql;
    /** @var list<array{kind:"pos"|"named",key:int|string}> */
    private array $placeholders;

    /** @var array<int|string,mixed> */
    private array $boundValues = [];
    /** @var array<int|string,mixed> */
    private array $boundParams = [];

    private bool $executed = false;
    private int $cursor = 0;
    /** @var list<array<string,mixed>> */
    private array $rows = [];
    /** @var list<array{name:string,native_type?:string}> */
    private array $columns = [];
    private int $rowCount = 0;
    private int $affectedRows = 0;
    private int $defaultFetchMode;
    /** @var list<mixed> */
    private array $defaultFetchModeArgs = [];
    /** @var array<int,mixed> */
    private array $attributes = [];
    /** @var array<int|string, mixed> */
    private array $boundColumns = [];
    /** @var array{0:string,1:int|string|null,2:string|null} */
    private array $errorInfo = ['00000', null, null];

    /**
     * @internal
     * @param list<array{kind:"pos"|"named",key:int|string}> $placeholders
     */
    public function __construct(PDO $pdo, string $sql, string $compiledSql, array $placeholders)
    {
        $this->pdo = $pdo;
        $this->sql = $sql;
        $this->compiledSql = $compiledSql;
        $this->placeholders = $placeholders;
        $this->queryString = $sql;
        $this->defaultFetchMode = $pdo->getAttribute(PDO::ATTR_DEFAULT_FETCH_MODE);
    }

    public function execute(?array $params = null): bool
    {
        $this->resetCursor();

        $final = $this->boundValues;
        foreach ($this->boundParams as $k => $v) {
            $final[$k] = $v;
        }
        if ($params !== null) {
            foreach ($params as $k => $v) {
                $final[Internal::normalizeParamKey(is_int($k) ? $k : (string)$k)] = $v;
            }
        }

        $ordered = [];
        foreach ($this->placeholders as $idx => $ph) {
            if ($ph['kind'] === 'named') {
                $key = (string)$ph['key'];
                $ordered[] = $final[$key] ?? null;
                continue;
            }

            $pos = (int)$ph['key']; // 1-based
            if (array_key_exists($pos, $final)) {
                $ordered[] = $final[$pos];
            } elseif (array_key_exists($pos - 1, $final)) {
                $ordered[] = $final[$pos - 1];
            } else {
                $ordered[] = null;
            }
        }

        try {
            [$rows, $columns, $affected, $lastInsertId] = $this->pdo->__internalExecuteCompiled(
                $this->compiledSql,
                $ordered,
                $this->isLikelyReturningRows($this->compiledSql)
            );
        } catch (\Throwable $e) {
            $this->errorInfo = $this->pdo->__internalRecordThrowable($e);
            return false;
        }

        $this->executed = true;
        $this->rows = $rows;
        $this->columns = $columns;
        $this->rowCount = is_array($rows) ? count($rows) : 0;
        $this->affectedRows = $affected;
        if ($lastInsertId !== null) {
            $this->pdo->__internalSetLastInsertId($lastInsertId);
        }
        return true;
    }

    /**
     * @param int|string $param
     */
    public function bindValue(int|string $param, mixed $value, int $type = PDO::PARAM_STR): bool
    {
        $key = Internal::normalizeParamKey($param);
        $this->boundValues[$key] = $value;
        return true;
    }

    /**
     * @param int|string $param
     */
    public function bindParam(int|string $param, mixed &$var, int $type = PDO::PARAM_STR, int $maxLength = 0, mixed $driverOptions = null): bool
    {
        $key = Internal::normalizeParamKey($param);
        $this->boundParams[$key] = &$var;
        return true;
    }

    public function fetch(int $mode = PDO::FETCH_DEFAULT): mixed
    {
        if (!$this->executed) {
            return false;
        }
        if ($this->cursor >= count($this->rows)) {
            return false;
        }
        $row = $this->rows[$this->cursor++];
        $mode = $mode === PDO::FETCH_DEFAULT ? $this->defaultFetchMode : $mode;

        if ($mode === PDO::FETCH_BOUND) {
            $this->applyBoundColumns($row);
            return true;
        }

        return $this->formatRow($row, $mode);
    }

    public function fetchAll(int $mode = PDO::FETCH_DEFAULT, mixed ...$args): array
    {
        if (!$this->executed) {
            return [];
        }
        $mode = $mode === PDO::FETCH_DEFAULT ? $this->defaultFetchMode : $mode;
        $args = $args !== [] ? $args : $this->defaultFetchModeArgs;

        if ($mode === PDO::FETCH_COLUMN) {
            $col = (int)($args[0] ?? 0);
            $out = [];
            while (true) {
                $v = $this->fetchColumn($col);
                if ($v === false) {
                    break;
                }
                $out[] = $v;
            }
            return $out;
        }

        if ($mode === PDO::FETCH_KEY_PAIR) {
            $out = [];
            while (true) {
                $row = $this->fetch(PDO::FETCH_NUM);
                if ($row === false) {
                    break;
                }
                $k = $row[0] ?? null;
                $v = $row[1] ?? null;
                $out[$k] = $v;
            }
            return $out;
        }

        $out = [];
        while (true) {
            $row = $this->fetch($mode);
            if ($row === false) {
                break;
            }
            $out[] = $row;
        }
        return $out;
    }

    public function fetchColumn(int $column = 0): mixed
    {
        $row = $this->fetch(PDO::FETCH_NUM);
        if ($row === false) {
            return false;
        }
        return $row[$column] ?? false;
    }

    public function fetchObject(string $class = "stdClass", ?array $constructorArgs = null): object|false
    {
        $row = $this->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            return false;
        }
        if ($class === 'stdClass') {
            return (object)$row;
        }

        $args = $constructorArgs ?? [];
        $obj = new $class(...$args);
        foreach ($row as $k => $v) {
            $obj->$k = $v;
        }
        return $obj;
    }

    public function rowCount(): int
    {
        return $this->affectedRows > 0 ? $this->affectedRows : $this->rowCount;
    }

    public function columnCount(): int
    {
        return count($this->columns);
    }

    public function errorCode(): string
    {
        return $this->errorInfo[0];
    }

    /**
     * @return array{0:string,1:int|string|null,2:string|null}
     */
    public function errorInfo(): array
    {
        return $this->errorInfo;
    }

    /**
     * @return array{name:string,native_type?:string}|false
     */
    public function getColumnMeta(int $column): array|false
    {
        return $this->columns[$column] ?? false;
    }

    /**
     * @param int|string $param
     */
    public function bindColumn(int|string $param, mixed &$var, int $type = PDO::PARAM_STR, int $maxLength = 0, mixed $driverOptions = null): bool
    {
        $key = Internal::normalizeParamKey($param);
        $this->boundColumns[$key] = &$var;
        return true;
    }

    public function closeCursor(): bool
    {
        $this->resetCursor();
        return true;
    }

    public function nextRowset(): bool
    {
        return false;
    }

    public function setFetchMode(int $mode, mixed ...$args): bool
    {
        $this->defaultFetchMode = $mode;
        $this->defaultFetchModeArgs = $args;
        return true;
    }

    public function setAttribute(int $attribute, mixed $value): bool
    {
        $this->attributes[$attribute] = $value;
        return true;
    }

    public function getAttribute(int $attribute): mixed
    {
        return $this->attributes[$attribute] ?? null;
    }

    public function debugDumpParams(): ?bool
    {
        $info = [
            'query' => $this->sql,
            'compiled' => $this->compiledSql,
            'placeholders' => $this->placeholders,
            'boundValues' => $this->boundValues,
        ];
        var_dump($info);
        return null;
    }

    public function getIterator(): \Traversable
    {
        return new \ArrayIterator($this->fetchAll());
    }

    private function resetCursor(): void
    {
        $this->cursor = 0;
        $this->rows = [];
        $this->columns = [];
        $this->rowCount = 0;
        $this->affectedRows = 0;
        $this->executed = false;
        $this->errorInfo = ['00000', null, null];
    }

    private function applyBoundColumns(array $row): void
    {
        if ($this->boundColumns === []) {
            return;
        }
        $num = array_values($row);
        foreach ($this->boundColumns as $k => &$ref) {
            if (is_int($k)) {
                $ref = $num[$k] ?? $num[$k - 1] ?? null;
            } else {
                $ref = $row[$k] ?? null;
            }
        }
    }

    private function formatRow(array $row, int $mode): mixed
    {
        return match ($mode) {
            PDO::FETCH_ASSOC => $row,
            PDO::FETCH_NUM => array_values($row),
            PDO::FETCH_BOTH => array_replace($row, array_values($row)),
            PDO::FETCH_OBJ => (object)$row,
            default => $row,
        };
    }

    private function isLikelyReturningRows(string $sql): bool
    {
        $s = ltrim($sql);
        $kw = strtoupper(strtok($s, " \t\r\n(") ?: '');
        if (in_array($kw, ['SELECT', 'SHOW', 'DESCRIBE', 'EXPLAIN', 'WITH', 'VALUES', 'CALL'], true)) {
            return true;
        }
        return stripos($sql, ' returning ') !== false;
    }
}
