<?php

namespace PDO;

use Async\Kernel\PDO\MySql as KernelMySql;
use Async\Kernel\PDO\PgSql as KernelPgSql;
use Async\Context as CoroutineContext;
use Fiber;

class PDO
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

    public const ATTR_CONNECTION_POOL_ENABLED = 10000;
    public const ATTR_CONNECTION_POOL_MIN_SIZE = 10001;
    public const ATTR_CONNECTION_POOL_MAX_SIZE = 10002;
    public const ATTR_CONNECTION_POOL_TIMEOUT = 10003;
    public const ATTR_CONNECTION_POOL_WAIT_TIMEOUT = 10004;
    public const ATTR_CONNECTION_POOL_HEARTBEAT = 10005;
    public const ATTR_CONNECTION_POOL_IDLE_TIME = 10006;

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
    private string $coroutineContextKeyPrefix;
    private string $connectionStatus = '';

    /** @var array<int,mixed> */
    private array $attributes = [];

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
        $this->attributes[self::ATTR_CURSOR] = (int)($options[self::ATTR_CURSOR] ?? self::CURSOR_FWDONLY);
        $this->attributes[self::ATTR_ORACLE_NULLS] = (int)($options[self::ATTR_ORACLE_NULLS] ?? self::NULL_NATURAL);
        $this->attributes[self::ATTR_STRINGIFY_FETCHES] = (bool)($options[self::ATTR_STRINGIFY_FETCHES] ?? false);

        $poolEnabled = (bool)($options[self::ATTR_CONNECTION_POOL_ENABLED] ?? false);
        $poolMaxSize = (int)($options[self::ATTR_CONNECTION_POOL_MAX_SIZE] ?? 0);
        if (!$poolEnabled && $poolMaxSize > 1) {
            $poolEnabled = true;
        }
        $poolMinSize = max(0, (int)($options[self::ATTR_CONNECTION_POOL_MIN_SIZE] ?? 0));
        $poolTimeout = $options[self::ATTR_CONNECTION_POOL_TIMEOUT] ?? null;
        $poolWaitTimeout = $options[self::ATTR_CONNECTION_POOL_WAIT_TIMEOUT] ?? null;
        $poolHeartbeat = $options[self::ATTR_CONNECTION_POOL_HEARTBEAT] ?? null;
        $poolIdleTime = $options[self::ATTR_CONNECTION_POOL_IDLE_TIME] ?? null;

        $result = pdo_dsn_to_sqlx($dsn, $username, $password);
        $driver = $result['driver'];
        $sqlxDsn = $result['uri'];
        $this->driverName = $driver;
        $this->connectionStatus = self::sanitizeConnectionStatus($sqlxDsn);
        $this->coroutineContextKeyPrefix = 'pdo:' . spl_object_id($this) . ':';

        $maxConns = 1;
        if ($poolEnabled) {
            $maxConns = $poolMaxSize > 0 ? $poolMaxSize : 10;
        }
        $maxConns = max(1, $maxConns);
        if ($poolMinSize > $maxConns) {
            $poolMinSize = $maxConns;
        }

        $minConns = ($poolEnabled && array_key_exists(self::ATTR_CONNECTION_POOL_MIN_SIZE, $options)) ? $poolMinSize : null;
        $connectTimeoutSecs = ($poolEnabled && array_key_exists(self::ATTR_CONNECTION_POOL_TIMEOUT, $options)) ? self::normalizeSeconds($poolTimeout) : null;
        $waitTimeoutSecs = ($poolEnabled && array_key_exists(self::ATTR_CONNECTION_POOL_WAIT_TIMEOUT, $options)) ? self::normalizeSeconds($poolWaitTimeout) : null;
        $idleTimeSecs = ($poolEnabled && array_key_exists(self::ATTR_CONNECTION_POOL_IDLE_TIME, $options)) ? self::normalizeSeconds($poolIdleTime) : null;
        $heartbeatEnabled = ($poolEnabled && array_key_exists(self::ATTR_CONNECTION_POOL_HEARTBEAT, $options))
            ? (self::normalizeSeconds($poolHeartbeat) !== null || (bool)$poolHeartbeat)
            : null;

        $this->attributes[self::ATTR_CONNECTION_POOL_ENABLED] = $poolEnabled;
        $this->attributes[self::ATTR_CONNECTION_POOL_MIN_SIZE] = $poolMinSize;
        $this->attributes[self::ATTR_CONNECTION_POOL_MAX_SIZE] = $maxConns;
        $this->attributes[self::ATTR_CONNECTION_POOL_TIMEOUT] = $poolTimeout;
        $this->attributes[self::ATTR_CONNECTION_POOL_WAIT_TIMEOUT] = $poolWaitTimeout;
        $this->attributes[self::ATTR_CONNECTION_POOL_HEARTBEAT] = $poolHeartbeat;
        $this->attributes[self::ATTR_CONNECTION_POOL_IDLE_TIME] = $poolIdleTime;

        try {
            $this->driver = $driver === 'mysql'
                ? Fiber::suspend(KernelMySql::connect($sqlxDsn, $maxConns, $minConns, $connectTimeoutSecs, $waitTimeoutSecs, $heartbeatEnabled, $idleTimeSecs))
                : Fiber::suspend(KernelPgSql::connect($sqlxDsn, $maxConns, $minConns, $connectTimeoutSecs, $waitTimeoutSecs, $heartbeatEnabled, $idleTimeSecs));
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
        $result = sql_compile_placeholders($this->driverName, $query);
        $compiled = $result['sql'];
        $placeholders = $result['placeholders'];
        $stmt = new PDOStatement($this, $query, $compiled, $placeholders);
        if (array_key_exists(self::ATTR_CURSOR, $options)) {
            $stmt->setAttribute(self::ATTR_CURSOR, (int)$options[self::ATTR_CURSOR]);
        }
        return $stmt;
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
            [, , $affected, $lastInsertId] = $this->__internalExecuteCompiled($statement, [], false);
            if ($lastInsertId !== null) {
                $this->setLastInsertId($lastInsertId);
            }
            return $affected;
        } catch (\Throwable $e) {
            $this->__internalRecordThrowable($e);
            return false;
        }
    }

    public function beginTransaction(): bool
    {
        if ($this->getInTransaction()) {
            return false;
        }
        try {
            $tx = Fiber::suspend($this->driver->beginTransaction());
            $this->setTx($tx);
            $this->setInTransaction($tx !== null);
            return $tx !== null;
        } catch (\Throwable $e) {
            $this->__internalRecordThrowable($e);
            return false;
        }
    }

    public function commit(): bool
    {
        $tx = $this->getTx();
        if (!$this->getInTransaction() || $tx === null) {
            return false;
        }
        try {
            $ok = Fiber::suspend($tx->commit());
            $this->setTx(null);
            $this->setInTransaction(false);
            return (bool)$ok;
        } catch (\Throwable $e) {
            $this->__internalRecordThrowable($e);
            return false;
        }
    }

    public function rollBack(): bool
    {
        $tx = $this->getTx();
        if (!$this->getInTransaction() || $tx === null) {
            return false;
        }
        try {
            $ok = Fiber::suspend($tx->rollback());
            $this->setTx(null);
            $this->setInTransaction(false);
            return (bool)$ok;
        } catch (\Throwable $e) {
            $this->__internalRecordThrowable($e);
            return false;
        }
    }

    public function inTransaction(): bool
    {
        return $this->getInTransaction();
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
        if ($attribute === self::ATTR_SERVER_VERSION) {
            return $this->getServerVersion();
        }
        if ($attribute === self::ATTR_CLIENT_VERSION) {
            return $this->getClientVersion();
        }
        if ($attribute === self::ATTR_SERVER_INFO) {
            return $this->getServerInfo();
        }
        if ($attribute === self::ATTR_CONNECTION_STATUS) {
            return $this->connectionStatus;
        }
        return $this->attributes[$attribute] ?? null;
    }

    public function errorCode(): string
    {
        return $this->getErrorInfo()[0];
    }

    /**
     * @return array{0:string,1:int|string|null,2:string|null}
     */
    public function errorInfo(): array
    {
        return $this->getErrorInfo();
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
        $lastInsertId = $this->getLastInsertId();
        if ($lastInsertId !== null && $name === null) {
            return $lastInsertId;
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
        $this->setErrorInfo([self::ERR_NONE, null, null]);

        $target = $this->getTx() ?? $this->driver;
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
        $this->setLastInsertId($id);
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
        $errorInfo = [$sqlstate, $code, $message];
        $this->setErrorInfo($errorInfo);

        $mode = (int)($this->attributes[self::ATTR_ERRMODE] ?? self::ERRMODE_EXCEPTION);
        if ($mode === self::ERRMODE_WARNING) {
            trigger_error("PDO[$sqlstate]: $message", E_USER_WARNING);
        } elseif ($mode === self::ERRMODE_EXCEPTION) {
            throw $this->asPdoException($sqlstate, $code, $message);
        }

        return $errorInfo;
    }

    private function asPdoException(string $sqlstate, int|string|null $code, string $message): PDOException
    {
        $e = new PDOException($message);
        $e->errorInfo = [$sqlstate, $code, $message];
        return $e;
    }

    private static function normalizeSeconds(mixed $value): ?float
    {
        if (is_int($value) || is_float($value)) {
            $secs = (float)$value;
            return $secs > 0.0 && is_finite($secs) ? $secs : null;
        }
        if (is_string($value) && $value !== '' && is_numeric($value)) {
            $secs = (float)$value;
            return $secs > 0.0 && is_finite($secs) ? $secs : null;
        }
        return null;
    }

    private static function sanitizeConnectionStatus(string $sqlxDsn): string
    {
        $parts = parse_url($sqlxDsn);
        if ($parts === false) {
            return $sqlxDsn;
        }

        $scheme = (string)($parts['scheme'] ?? '');
        $host = (string)($parts['host'] ?? '');
        $port = $parts['port'] ?? null;
        $path = (string)($parts['path'] ?? '');
        $user = (string)($parts['user'] ?? '');

        $out = '';
        if ($scheme !== '') {
            $out .= $scheme . '://';
        }
        if ($user !== '') {
            $out .= $user . '@';
        }
        $out .= $host;
        if ($port !== null && $port !== '') {
            $out .= ':' . (int)$port;
        }
        $out .= $path;

        $query = (string)($parts['query'] ?? '');
        if ($query !== '') {
            parse_str($query, $q);
            if (is_array($q)) {
                unset($q['password'], $q['pass']);
                if ($q !== []) {
                    $out .= '?' . http_build_query($q, '', '&', PHP_QUERY_RFC3986);
                }
            }
        }

        return $out;
    }

    private function getServerVersion(): string|false
    {
        $cached = $this->attributes[self::ATTR_SERVER_VERSION] ?? null;
        if (is_string($cached) && $cached !== '') {
            return $cached;
        }

        try {
            if ($this->driverName === 'mysql') {
                $stmt = $this->query('SELECT VERSION() AS v');
                if ($stmt === false) return false;
                $v = $stmt->fetchColumn(0);
                if ($v === false) return false;
                $this->attributes[self::ATTR_SERVER_VERSION] = (string)$v;
                return (string)$v;
            }

            $stmt = $this->query("SHOW server_version");
            if ($stmt === false) return false;
            $v = $stmt->fetchColumn(0);
            if ($v === false) return false;
            $this->attributes[self::ATTR_SERVER_VERSION] = (string)$v;
            return (string)$v;
        } catch (\Throwable) {
            return false;
        }
    }

    private function getClientVersion(): string|false
    {
        $cached = $this->attributes[self::ATTR_CLIENT_VERSION] ?? null;
        if (is_string($cached) && $cached !== '') {
            return $cached;
        }

        $v = 'async-php/sqlx';
        $this->attributes[self::ATTR_CLIENT_VERSION] = $v;
        return $v;
    }

    public function pgsqlTransactionStatus(): int
    {
        if ($this->driverName !== 'pgsql') {
            return self::PGSQL_TRANSACTION_UNKNOWN;
        }
        if ($this->getInTransaction()) {
            return $this->errorCode() === self::ERR_NONE
                ? self::PGSQL_TRANSACTION_INTRANS
                : self::PGSQL_TRANSACTION_INERROR;
        }
        return self::PGSQL_TRANSACTION_IDLE;
    }

    private function getServerInfo(): string|false
    {
        $cached = $this->attributes[self::ATTR_SERVER_INFO] ?? null;
        if (is_string($cached) && $cached !== '') {
            return $cached;
        }

        try {
            if ($this->driverName === 'mysql') {
                $stmt = $this->query("SELECT CONCAT(VERSION(), ' ', @@version_comment) AS v");
                if ($stmt === false) return false;
                $v = $stmt->fetchColumn(0);
                if ($v === false) return false;
                $this->attributes[self::ATTR_SERVER_INFO] = (string)$v;
                return (string)$v;
            }

            $stmt = $this->query('SELECT version() AS v');
            if ($stmt === false) return false;
            $v = $stmt->fetchColumn(0);
            if ($v === false) return false;
            $this->attributes[self::ATTR_SERVER_INFO] = (string)$v;
            return (string)$v;
        } catch (\Throwable) {
            return false;
        }
    }

    private function ctxKey(string $suffix): string
    {
        return $this->coroutineContextKeyPrefix . $suffix;
    }

    private function getTx(): ?object
    {
        $tx = CoroutineContext::get($this->ctxKey('tx'), null);
        return is_object($tx) ? $tx : null;
    }

    private function setTx(?object $tx): void
    {
        CoroutineContext::set($this->ctxKey('tx'), $tx);
    }

    private function getInTransaction(): bool
    {
        return (bool)CoroutineContext::get($this->ctxKey('in_tx'), false);
    }

    private function setInTransaction(bool $inTransaction): void
    {
        CoroutineContext::set($this->ctxKey('in_tx'), $inTransaction);
    }

    /**
     * @return array{0:string,1:int|string|null,2:string|null}
     */
    private function getErrorInfo(): array
    {
        $errorInfo = CoroutineContext::get($this->ctxKey('error_info'), null);
        if (
            is_array($errorInfo)
            && array_key_exists(0, $errorInfo)
            && array_key_exists(1, $errorInfo)
            && array_key_exists(2, $errorInfo)
            && is_string($errorInfo[0])
        ) {
            /** @var array{0:string,1:int|string|null,2:string|null} $errorInfo */
            return $errorInfo;
        }
        return [self::ERR_NONE, null, null];
    }

    /**
     * @param array{0:string,1:int|string|null,2:string|null} $errorInfo
     */
    private function setErrorInfo(array $errorInfo): void
    {
        CoroutineContext::set($this->ctxKey('error_info'), $errorInfo);
    }

    private function getLastInsertId(): ?string
    {
        $id = CoroutineContext::get($this->ctxKey('last_insert_id'), null);
        return is_string($id) ? $id : null;
    }

    private function setLastInsertId(?string $id): void
    {
        CoroutineContext::set($this->ctxKey('last_insert_id'), $id);
    }
}
