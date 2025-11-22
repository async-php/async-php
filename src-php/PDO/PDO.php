<?php

namespace PDO;

use Async\Kernel\PDO\MySql as KernelMySql;
use Async\Kernel\PDO\PgSql as KernelPgSql;
use Fiber;

final class PDO
{
    // Generic PDO constants (matching ext/pdo values on PHP 8.4)
    public const ATTR_AUTOCOMMIT = 0;
    public const ATTR_PREFETCH = 1;
    public const ATTR_TIMEOUT = 2;
    public const ATTR_ERRMODE = 3;
    public const ATTR_SERVER_VERSION = 4;
    public const ATTR_CLIENT_VERSION = 5;
    public const ATTR_SERVER_INFO = 6;
    public const ATTR_CONNECTION_STATUS = 7;
    public const ATTR_CASE = 8;
    public const ATTR_CURSOR_NAME = 9;
    public const ATTR_CURSOR = 10;
    public const ATTR_ORACLE_NULLS = 11;
    public const ATTR_PERSISTENT = 12;
    public const ATTR_STATEMENT_CLASS = 13;
    public const ATTR_FETCH_TABLE_NAMES = 14;
    public const ATTR_FETCH_CATALOG_NAMES = 15;
    public const ATTR_DRIVER_NAME = 16;
    public const ATTR_STRINGIFY_FETCHES = 17;
    public const ATTR_MAX_COLUMN_LEN = 18;
    public const ATTR_DEFAULT_FETCH_MODE = 19;
    public const ATTR_EMULATE_PREPARES = 20;
    public const ATTR_DEFAULT_STR_PARAM = 21;

    public const ERRMODE_SILENT = 0;
    public const ERRMODE_WARNING = 1;
    public const ERRMODE_EXCEPTION = 2;

    public const CASE_NATURAL = 0;
    public const CASE_UPPER = 1;
    public const CASE_LOWER = 2;

    public const CURSOR_FWDONLY = 0;
    public const CURSOR_SCROLL = 1;

    public const NULL_NATURAL = 0;
    public const NULL_EMPTY_STRING = 1;
    public const NULL_TO_STRING = 2;

    public const FETCH_DEFAULT = 0;
    public const FETCH_LAZY = 1;
    public const FETCH_ASSOC = 2;
    public const FETCH_NUM = 3;
    public const FETCH_BOTH = 4;
    public const FETCH_OBJ = 5;
    public const FETCH_BOUND = 6;
    public const FETCH_COLUMN = 7;
    public const FETCH_CLASS = 8;
    public const FETCH_INTO = 9;
    public const FETCH_FUNC = 10;
    public const FETCH_NAMED = 11;
    public const FETCH_KEY_PAIR = 12;

    public const FETCH_GROUP = 65536;
    public const FETCH_UNIQUE = 196608;
    public const FETCH_CLASSTYPE = 262144;
    public const FETCH_SERIALIZE = 524288;
    public const FETCH_PROPS_LATE = 1048576;

    public const FETCH_ORI_NEXT = 0;
    public const FETCH_ORI_PRIOR = 1;
    public const FETCH_ORI_FIRST = 2;
    public const FETCH_ORI_LAST = 3;
    public const FETCH_ORI_ABS = 4;
    public const FETCH_ORI_REL = 5;

    public const PARAM_NULL = 0;
    public const PARAM_INT = 1;
    public const PARAM_STR = 2;
    public const PARAM_LOB = 3;
    public const PARAM_STMT = 4;
    public const PARAM_BOOL = 5;
    public const PARAM_INPUT_OUTPUT = 2147483648;

    public const ERR_NONE = '00000';

    // MySQL driver attrs (subset)
    public const MYSQL_ATTR_USE_BUFFERED_QUERY = 1000;
    public const MYSQL_ATTR_LOCAL_INFILE = 1001;
    public const MYSQL_ATTR_INIT_COMMAND = 1002;
    public const MYSQL_ATTR_COMPRESS = 1003;
    public const MYSQL_ATTR_DIRECT_QUERY = 1004;
    public const MYSQL_ATTR_FOUND_ROWS = 1005;
    public const MYSQL_ATTR_IGNORE_SPACE = 1006;
    public const MYSQL_ATTR_SSL_KEY = 1007;
    public const MYSQL_ATTR_SSL_CERT = 1008;
    public const MYSQL_ATTR_SSL_CA = 1009;
    public const MYSQL_ATTR_SSL_CAPATH = 1010;
    public const MYSQL_ATTR_SSL_CIPHER = 1011;
    public const MYSQL_ATTR_SERVER_PUBLIC_KEY = 1012;
    public const MYSQL_ATTR_MULTI_STATEMENTS = 1013;
    public const MYSQL_ATTR_SSL_VERIFY_SERVER_CERT = 1014;
    public const MYSQL_ATTR_LOCAL_INFILE_DIRECTORY = 1015;

    // PgSQL driver attrs (subset)
    public const PGSQL_ATTR_DISABLE_PREPARES = 1000;
    public const PGSQL_TRANSACTION_IDLE = 0;
    public const PGSQL_TRANSACTION_ACTIVE = 1;
    public const PGSQL_TRANSACTION_INTRANS = 2;
    public const PGSQL_TRANSACTION_INERROR = 3;
    public const PGSQL_TRANSACTION_UNKNOWN = 4;

    private string $driverName;
    private KernelMySql|KernelPgSql $driver;
    private object|null $tx = null;
    private bool $inTransaction = false;

    /** @var array<int,mixed> */
    private array $attributes = [];
    /** @var array{0:string,1:int|string|null,2:string|null} */
    private array $errorInfo = [self::ERR_NONE, null, null];
    private ?string $lastInsertId = null;

    public function __construct(string $dsn, ?string $username = null, ?string $password = null, ?array $options = null)
    {
        $options = $options ?? [];

        if (isset($options[self::ATTR_ERRMODE])) {
            $this->attributes[self::ATTR_ERRMODE] = (int)$options[self::ATTR_ERRMODE];
        } else {
            $this->attributes[self::ATTR_ERRMODE] = self::ERRMODE_EXCEPTION;
        }
        $this->attributes[self::ATTR_DEFAULT_FETCH_MODE] = (int)($options[self::ATTR_DEFAULT_FETCH_MODE] ?? self::FETCH_BOTH);
        $this->attributes[self::ATTR_EMULATE_PREPARES] = (bool)($options[self::ATTR_EMULATE_PREPARES] ?? true);
        $this->attributes[self::ATTR_CASE] = (int)($options[self::ATTR_CASE] ?? self::CASE_NATURAL);
        $this->attributes[self::ATTR_ORACLE_NULLS] = (int)($options[self::ATTR_ORACLE_NULLS] ?? self::NULL_NATURAL);
        $this->attributes[self::ATTR_STRINGIFY_FETCHES] = (bool)($options[self::ATTR_STRINGIFY_FETCHES] ?? false);

        [$driver, $sqlxDsn] = Internal::dsnToSqlx($dsn, $username, $password);
        $this->driverName = $driver;

        try {
            $this->driver = $driver === 'mysql'
                ? Fiber::suspend(KernelMySql::connect($sqlxDsn, 1))
                : Fiber::suspend(KernelPgSql::connect($sqlxDsn, 1));
        } catch (\Throwable $e) {
            throw $this->asPdoException('HY000', null, $e->getMessage());
        }
    }

    public static function connect(string $dsn, ?string $username = null, ?string $password = null, ?array $options = null): static
    {
        return new static($dsn, $username, $password, $options);
    }

    public static function getAvailableDrivers(): array
    {
        return ['mysql', 'pgsql'];
    }

    public function prepare(string $query, array $options = []): PDOStatement|false
    {
        [$compiled, $placeholders] = Internal::compilePlaceholders($this->driverName, $query);
        return new PDOStatement($this, $query, $compiled, $placeholders);
    }

    public function query(string $query, ?int $fetchMode = null, mixed ...$fetchModeArgs): PDOStatement|false
    {
        $stmt = $this->prepare($query);
        if ($stmt === false) {
            return false;
        }
        if ($fetchMode !== null) {
            $stmt->setFetchMode($fetchMode, ...$fetchModeArgs);
        }
        if (!$stmt->execute()) {
            return false;
        }
        return $stmt;
    }

    public function exec(string $statement): int|false
    {
        try {
            [, , $affected] = $this->__internalExecuteCompiled($statement, [], false);
            return $affected;
        } catch (\Throwable $e) {
            $this->__internalRecordThrowable($e);
            return false;
        }
    }

    public function beginTransaction(): bool
    {
        if ($this->inTransaction) {
            return false;
        }
        try {
            $this->tx = Fiber::suspend($this->driver->beginTransaction());
            $this->inTransaction = $this->tx !== null;
            return $this->inTransaction;
        } catch (\Throwable $e) {
            $this->__internalRecordThrowable($e);
            return false;
        }
    }

    public function commit(): bool
    {
        if (!$this->inTransaction || $this->tx === null) {
            return false;
        }
        try {
            $ok = Fiber::suspend($this->tx->commit());
            $this->tx = null;
            $this->inTransaction = false;
            return (bool)$ok;
        } catch (\Throwable $e) {
            $this->__internalRecordThrowable($e);
            return false;
        }
    }

    public function rollBack(): bool
    {
        if (!$this->inTransaction || $this->tx === null) {
            return false;
        }
        try {
            $ok = Fiber::suspend($this->tx->rollback());
            $this->tx = null;
            $this->inTransaction = false;
            return (bool)$ok;
        } catch (\Throwable $e) {
            $this->__internalRecordThrowable($e);
            return false;
        }
    }

    public function inTransaction(): bool
    {
        return $this->inTransaction;
    }

    public function setAttribute(int $attribute, mixed $value): bool
    {
        if ($attribute === self::ATTR_DRIVER_NAME) {
            return false;
        }
        $this->attributes[$attribute] = $value;
        return true;
    }

    public function getAttribute(int $attribute): mixed
    {
        if ($attribute === self::ATTR_DRIVER_NAME) {
            return $this->driverName;
        }
        return $this->attributes[$attribute] ?? null;
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

    public function quote(string $string, int $type = self::PARAM_STR): string|false
    {
        if ($type !== self::PARAM_STR) {
            return false;
        }
        return "'" . str_replace("'", "''", $string) . "'";
    }

    public function lastInsertId(?string $name = null): string|false
    {
        if ($this->lastInsertId !== null && $name === null) {
            return $this->lastInsertId;
        }

        try {
            if ($this->driverName === 'mysql') {
                $stmt = $this->query('SELECT LAST_INSERT_ID() AS id');
                if ($stmt === false) return false;
                $id = $stmt->fetchColumn(0);
                return $id === false ? false : (string)$id;
            }

            if ($name !== null && $name !== '') {
                $q = 'SELECT CURRVAL(' . $this->quote($name) . ') AS id';
            } else {
                $q = 'SELECT LASTVAL() AS id';
            }
            $stmt = $this->query($q);
            if ($stmt === false) return false;
            $id = $stmt->fetchColumn(0);
            return $id === false ? false : (string)$id;
        } catch (\Throwable $e) {
            $this->__internalRecordThrowable($e);
            return false;
        }
    }

    /**
     * @internal
     * @return array{0:list<array<string,mixed>>,1:list<array{name:string,native_type?:string}>,2:int,3:?string}
     */
    public function __internalExecuteCompiled(string $compiledSql, array $orderedParams, bool $returnsRows): array
    {
        $this->errorInfo = [self::ERR_NONE, null, null];

        $target = $this->tx ?? $this->driver;
        $params = $orderedParams === [] ? null : $orderedParams;

        if ($returnsRows) {
            /** @var array{rows:list<array<string,mixed>>,columns:list<array{name:string,native_type?:string}>} $res */
            $res = Fiber::suspend($target->query($compiledSql, $params));
            $rows = $res['rows'] ?? [];
            $columns = $res['columns'] ?? $this->inferColumnsFromRows($rows);
            return [$rows, $columns, 0, null];
        }

        /** @var array{rows_affected:int,last_insert_id?:string|null} $res */
        $res = Fiber::suspend($target->execute($compiledSql, $params));
        $affected = (int)($res['rows_affected'] ?? 0);
        $lastInsertId = $res['last_insert_id'] ?? null;
        return [[], [], $affected, $lastInsertId];
    }

    /**
     * @internal
     * @return array{0:string,1:int|string|null,2:string|null}
     */
    public function __internalRecordThrowable(\Throwable $e): array
    {
        $msg = $e->getMessage();
        if (preg_match('/^SQLSTATE\\[([0-9A-Z]{5})\\]:\\s*(.*)$/', $msg, $m) === 1) {
            return $this->recordError($m[1], null, $m[2]);
        }
        return $this->recordError('HY000', null, $msg);
    }

    /**
     * @internal
     */
    public function __internalSetLastInsertId(string $id): void
    {
        $this->lastInsertId = $id;
    }

    /**
     * @param list<array<string,mixed>> $rows
     * @return list<array{name:string}>
     */
    private function inferColumnsFromRows(array $rows): array
    {
        if ($rows === []) {
            return [];
        }
        $first = $rows[0];
        $cols = [];
        foreach (array_keys($first) as $name) {
            if (is_string($name)) {
                $cols[] = ['name' => $name];
            }
        }
        return $cols;
    }

    /**
     * @return array{0:string,1:int|string|null,2:string|null}
     */
    private function recordError(string $sqlstate, int|string|null $code, string $message): array
    {
        $this->errorInfo = [$sqlstate, $code, $message];

        $mode = (int)($this->attributes[self::ATTR_ERRMODE] ?? self::ERRMODE_EXCEPTION);
        if ($mode === self::ERRMODE_WARNING) {
            trigger_error("PDO[$sqlstate]: $message", E_USER_WARNING);
        } elseif ($mode === self::ERRMODE_EXCEPTION) {
            throw $this->asPdoException($sqlstate, $code, $message);
        }

        return $this->errorInfo;
    }

    private function asPdoException(string $sqlstate, int|string|null $code, string $message): PDOException
    {
        $e = new PDOException($message);
        $e->errorInfo = [$sqlstate, $code, $message];
        return $e;
    }
}
