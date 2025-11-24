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
    /** @var array<int|string,array{ref:mixed,type:int}> */
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
        $this->attributes[PDO::ATTR_CURSOR] = (int)$pdo->getAttribute(PDO::ATTR_CURSOR);
    }

    public function execute(?array $params = null): bool
    {
        $this->resetCursor();

        $final = $this->boundValues;
        foreach ($this->boundParams as $k => $v) {
            $final[$k] = $this->coerceParamValue($v['ref'], $v['type']);
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

            // For positional parameters, $ph['key'] is 1-based from the parser
            $pos = (int)$ph['key']; // 1-based
            // But PHP arrays are 0-based, so check 0-based first
            if (array_key_exists($pos - 1, $final)) {
                $ordered[] = $final[$pos - 1];
            } elseif (array_key_exists($pos, $final)) {
                $ordered[] = $final[$pos];
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
        $this->boundValues[$key] = $this->coerceParamValue($value, $type);
        return true;
    }

    /**
     * @param int|string $param
     */
    public function bindParam(int|string $param, mixed &$var, int $type = PDO::PARAM_STR, int $maxLength = 0, mixed $driverOptions = null): bool
    {
        $key = Internal::normalizeParamKey($param);
        $this->boundParams[$key] = ['ref' => &$var, 'type' => $type];
        return true;
    }

    public function fetch(int $mode = PDO::FETCH_DEFAULT, int $cursorOrientation = PDO::FETCH_ORI_NEXT, int $cursorOffset = 0): mixed
    {
        if (!$this->executed) {
            return false;
        }
        if ($cursorOrientation !== PDO::FETCH_ORI_NEXT && !$this->isCursorScrollable()) {
            return false;
        }

        $idx = $this->resolveCursorIndex($cursorOrientation, $cursorOffset);
        if ($idx === null) {
            return false;
        }
        $row = $this->rows[$idx];
        $this->cursor = $idx + 1;

        $args = [];
        if ($mode === PDO::FETCH_DEFAULT) {
            $mode = $this->defaultFetchMode;
            $args = $this->defaultFetchModeArgs;
        }

        if ($mode === PDO::FETCH_BOUND) {
            $this->applyBoundColumns($row);
            return true;
        }

        return $this->formatRowWithArgs($row, $mode, $args);
    }

    public function fetchAll(int $mode = PDO::FETCH_DEFAULT, mixed ...$args): array
    {
        if (!$this->executed) {
            return [];
        }
        if ($mode === PDO::FETCH_DEFAULT) {
            $mode = $this->defaultFetchMode;
            $args = $args !== [] ? $args : $this->defaultFetchModeArgs;
        }

        $group = ($mode & PDO::FETCH_GROUP) === PDO::FETCH_GROUP;
        $unique = ($mode & PDO::FETCH_UNIQUE) === PDO::FETCH_UNIQUE;
        $flags = PDO::FETCH_GROUP | PDO::FETCH_UNIQUE | PDO::FETCH_CLASSTYPE | PDO::FETCH_SERIALIZE | PDO::FETCH_PROPS_LATE;
        $baseMode = $mode & (~$flags);
        if ($baseMode === 0) {
            $baseMode = PDO::FETCH_BOTH;
        }

        if ($group || $unique) {
            $this->cursor = count($this->rows);
            return $this->fetchAllGroupedOrUnique($baseMode, $args, $group, $unique, $mode);
        }

        if ($baseMode === PDO::FETCH_COLUMN) {
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

        if ($baseMode === PDO::FETCH_KEY_PAIR) {
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

        if ($baseMode === PDO::FETCH_INTO) {
            $into = $args[0] ?? null;
            if (!is_object($into)) {
                return [];
            }
            $caseMode = (int)$this->pdo->getAttribute(PDO::ATTR_CASE);
            $stringify = (bool)$this->pdo->getAttribute(PDO::ATTR_STRINGIFY_FETCHES);
            $out = [];
            foreach ($this->rows as $rawRow) {
                try {
                    $obj = clone $into;
                } catch (\Throwable) {
                    $obj = $into;
                }
                $this->assignRowToObject($rawRow, $obj, $caseMode, $stringify);
                $out[] = $obj;
            }
            $this->cursor = count($this->rows);
            return $out;
        }

        $out = [];
        foreach ($this->rows as $rawRow) {
            $out[] = $this->formatRowWithArgs($rawRow, $mode, $args);
        }
        $this->cursor = count($this->rows);
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

    private function isCursorScrollable(): bool
    {
        $v = $this->getAttribute(PDO::ATTR_CURSOR);
        if (is_int($v)) {
            return $v === PDO::CURSOR_SCROLL;
        }
        return false;
    }

    private function resolveCursorIndex(int $orientation, int $offset): ?int
    {
        $count = count($this->rows);
        if ($count === 0) {
            return null;
        }

        $idx = null;
        if ($orientation === PDO::FETCH_ORI_NEXT) {
            $idx = $this->cursor;
        } elseif ($orientation === PDO::FETCH_ORI_PRIOR) {
            $idx = $this->cursor - 2;
        } elseif ($orientation === PDO::FETCH_ORI_FIRST) {
            $idx = 0;
        } elseif ($orientation === PDO::FETCH_ORI_LAST) {
            $idx = $count - 1;
        } elseif ($orientation === PDO::FETCH_ORI_ABS) {
            $idx = $offset;
        } elseif ($orientation === PDO::FETCH_ORI_REL) {
            $idx = $this->cursor + $offset;
        } else {
            return null;
        }

        if (!is_int($idx) || $idx < 0 || $idx >= $count) {
            return null;
        }
        return $idx;
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

    /**
     * @param list<mixed> $args
     */
    private function formatRowWithArgs(array $row, int $mode, array $args): mixed
    {
        $caseMode = (int)$this->pdo->getAttribute(PDO::ATTR_CASE);
        $stringify = (bool)$this->pdo->getAttribute(PDO::ATTR_STRINGIFY_FETCHES);

        $flags = PDO::FETCH_GROUP | PDO::FETCH_UNIQUE | PDO::FETCH_CLASSTYPE | PDO::FETCH_SERIALIZE | PDO::FETCH_PROPS_LATE;
        $baseMode = $mode & (~$flags);
        if ($baseMode === 0) {
            $baseMode = PDO::FETCH_BOTH;
        }

        if ($baseMode === PDO::FETCH_NAMED) {
            $baseMode = PDO::FETCH_ASSOC;
        }

        if ($baseMode === PDO::FETCH_CLASS) {
            $class = (string)($args[0] ?? 'stdClass');
            $ctorArgs = $args[1] ?? null;
            if ($ctorArgs !== null && !is_array($ctorArgs)) {
                $ctorArgs = null;
            }
            $propsLate = ($mode & PDO::FETCH_PROPS_LATE) === PDO::FETCH_PROPS_LATE;
            $classType = ($mode & PDO::FETCH_CLASSTYPE) === PDO::FETCH_CLASSTYPE;
            if ($classType) {
                $vals = array_values($row);
                $class = isset($vals[0]) ? (string)$vals[0] : $class;
            }
            return $this->rowToClassObject($row, $class, $ctorArgs ?? [], $propsLate, $classType, $caseMode, $stringify);
        }

        if ($baseMode === PDO::FETCH_INTO) {
            $into = $args[0] ?? null;
            if (!is_object($into)) {
                return false;
            }
            $this->assignRowToObject($row, $into, $caseMode, $stringify);
            return $into;
        }

        if ($baseMode === PDO::FETCH_FUNC) {
            $fn = $args[0] ?? null;
            if (!is_callable($fn)) {
                return false;
            }
            $vals = array_values($this->stringifyRowValues($row, $stringify));
            return $fn(...$vals);
        }

        $row = $this->applyCaseMode($row, $caseMode);
        $row = $this->stringifyRowValues($row, $stringify);

        return match ($baseMode) {
            PDO::FETCH_ASSOC => $row,
            PDO::FETCH_NUM => array_values($row),
            PDO::FETCH_BOTH => array_replace($row, array_values($row)),
            PDO::FETCH_OBJ => (object)$row,
            default => $row,
        };
    }

    private function applyCaseMode(array $row, int $caseMode): array
    {
        if ($caseMode !== PDO::CASE_UPPER && $caseMode !== PDO::CASE_LOWER) {
            return $row;
        }
        $out = [];
        foreach ($row as $k => $v) {
            if (is_string($k)) {
                $k = $caseMode === PDO::CASE_UPPER ? strtoupper($k) : strtolower($k);
            }
            $out[$k] = $v;
        }
        return $out;
    }

    private function stringifyRowValues(array $row, bool $stringify): array
    {
        if (!$stringify) {
            return $row;
        }
        foreach ($row as $k => $v) {
            if ($v === null || is_string($v)) {
                continue;
            }
            if (is_bool($v)) {
                $row[$k] = $v ? '1' : '0';
                continue;
            }
            if (is_int($v) || is_float($v)) {
                $row[$k] = (string)$v;
            }
        }
        return $row;
    }

    /**
     * @param list<mixed> $ctorArgs
     */
    private function rowToClassObject(array $row, string $class, array $ctorArgs, bool $propsLate, bool $classType, int $caseMode, bool $stringify): object|false
    {
        if ($class === '') {
            $class = 'stdClass';
        }
        if ($class === 'stdClass') {
            $row = $this->applyCaseMode($row, $caseMode);
            $row = $this->stringifyRowValues($row, $stringify);
            $obj = new \stdClass();
            foreach ($row as $k => $v) {
                if (is_string($k)) {
                    $obj->$k = $v;
                }
            }
            return $obj;
        }

        try {
            $rc = new \ReflectionClass($class);

            $firstName = $this->columns[0]['name'] ?? null;

            if ($propsLate) {
                $obj = $rc->newInstanceArgs($ctorArgs);
                $this->assignRowToObject($row, $obj, $caseMode, $stringify, $classType, $firstName);
                return $obj;
            }

            $obj = $rc->newInstanceWithoutConstructor();
            $this->assignRowToObject($row, $obj, $caseMode, $stringify, $classType, $firstName);
            $ctor = $rc->getConstructor();
            if ($ctor !== null && $ctor->isPublic()) {
                $ctor->invokeArgs($obj, $ctorArgs);
            }
            return $obj;
        } catch (\Throwable) {
            return false;
        }
    }

    private function assignRowToObject(array $row, object $obj, int $caseMode, bool $stringify, bool $classType = false, ?string $classTypeColumnName = null): void
    {
        $row = $this->applyCaseMode($row, $caseMode);
        $row = $this->stringifyRowValues($row, $stringify);

        foreach ($row as $k => $v) {
            if (!is_string($k)) {
                continue;
            }
            if ($classType && $classTypeColumnName !== null && $k === $classTypeColumnName) {
                continue;
            }
            $obj->$k = $v;
        }
    }

    /**
     * @param list<mixed> $args
     */
    private function fetchAllGroupedOrUnique(int $baseMode, array $args, bool $group, bool $unique, int $fullMode): array
    {
        $out = [];
        $firstName = $this->columns[0]['name'] ?? null;

        foreach ($this->rows as $rawRow) {
            $vals = array_values($rawRow);
            $key = $vals[0] ?? null;

            $row = $this->formatRowWithArgs($rawRow, $fullMode, $args);
            $row = $this->removeFirstColumnFromFormattedRow($row, $baseMode, $firstName);

            if ($group) {
                if (!array_key_exists($key, $out)) {
                    $out[$key] = [];
                }
                $out[$key][] = $row;
            } elseif ($unique) {
                $out[$key] = $row;
            }
        }

        return $out;
    }

    private function removeFirstColumnFromFormattedRow(mixed $row, int $baseMode, ?string $firstName): mixed
    {
        if ($row === false || $row === null) {
            return $row;
        }

        if ($baseMode === PDO::FETCH_NUM && is_array($row)) {
            array_shift($row);
            return $row;
        }

        if (($baseMode === PDO::FETCH_ASSOC || $baseMode === PDO::FETCH_BOTH) && is_array($row)) {
            if ($firstName !== null && $firstName !== '') {
                unset($row[$firstName]);
                $caseMode = (int)$this->pdo->getAttribute(PDO::ATTR_CASE);
                if ($caseMode === PDO::CASE_UPPER) {
                    unset($row[strtoupper($firstName)]);
                } elseif ($caseMode === PDO::CASE_LOWER) {
                    unset($row[strtolower($firstName)]);
                }
            }
            unset($row[0]);
            return $row;
        }

        return $row;
    }

    private function coerceParamValue(mixed $value, int $type): mixed
    {
        $type = $type & (~PDO::PARAM_INPUT_OUTPUT);

        return match ($type) {
            PDO::PARAM_NULL => null,
            PDO::PARAM_INT => $value === null ? null : (int)$value,
            PDO::PARAM_BOOL => $value === null ? null : (bool)$value,
            PDO::PARAM_STR => $value === null ? null : (string)$value,
            PDO::PARAM_LOB => $this->coerceLob($value),
            default => $value,
        };
    }

    private function coerceLob(mixed $value): mixed
    {
        if ($value === null) {
            return null;
        }
        if (is_resource($value) && get_resource_type($value) === 'stream') {
            $data = stream_get_contents($value);
            return $data === false ? null : $data;
        }
        if (is_string($value)) {
            return $value;
        }
        return (string)$value;
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
