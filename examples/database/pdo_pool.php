<?php

require __DIR__ . '/../../vendor/autoload.php';

use Async\Channel;
use Async\Kernel;

error_reporting(E_ALL);
ini_set('display_errors', '1');

function ok(string $label, mixed $value = null): void
{
    echo "[OK] $label";
    if (func_num_args() > 1) {
        echo ": " . json_encode($value, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE) . PHP_EOL;
        return;
    }
    echo PHP_EOL;
}

function fail(string $label, Throwable $e): void
{
    echo "[FAIL] $label: " . $e->getMessage() . PHP_EOL;
    throw $e;
}

/**
 * Env overrides:
 * - MYSQL_DSN / MYSQL_USER / MYSQL_PASS
 * - PGSQL_DSN / PGSQL_USER / PGSQL_PASS
 */
$driver = $argv[1] ?? 'mysql';
$driver = strtolower(trim($driver));
if (!in_array($driver, ['mysql', 'pgsql'], true)) {
    fwrite(STDERR, "Usage: php examples/pdo_pool_test.php [mysql|pgsql]\n");
    exit(2);
}

$defaults = [
    'mysql' => [
        'dsn' => 'mysql:host=127.0.0.1;port=3306;dbname=mysql',
        'user' => 'root',
        'pass' => 'root',
    ],
    'pgsql' => [
        'dsn' => 'pgsql:host=127.0.0.1;port=5432;dbname=postgres',
        'user' => 'postgres',
        'pass' => 'postgres',
    ],
];

$dsn = getenv(strtoupper($driver) . '_DSN') ?: $defaults[$driver]['dsn'];
$user = getenv(strtoupper($driver) . '_USER') ?: $defaults[$driver]['user'];
$pass = getenv(strtoupper($driver) . '_PASS') ?: $defaults[$driver]['pass'];

Kernel::run(function () use ($driver, $dsn, $user, $pass) {
    $poolMax = 5;
    $taskN = 20;

    try {
        $pdo = new \PDO\PDO(
            $dsn,
            $user,
            $pass,
            [
                \PDO\PDO::ATTR_ERRMODE => \PDO\PDO::ERRMODE_EXCEPTION,
                \PDO\PDO::ATTR_DEFAULT_FETCH_MODE => \PDO\PDO::FETCH_ASSOC,
                \PDO\PDO::ATTR_CONNECTION_POOL_ENABLED => true,
                \PDO\PDO::ATTR_CONNECTION_POOL_MIN_SIZE => 1,
                \PDO\PDO::ATTR_CONNECTION_POOL_MAX_SIZE => $poolMax,
                \PDO\PDO::ATTR_CONNECTION_POOL_TIMEOUT => 5,
                \PDO\PDO::ATTR_CONNECTION_POOL_WAIT_TIMEOUT => 5,
                \PDO\PDO::ATTR_CONNECTION_POOL_HEARTBEAT => true,
                \PDO\PDO::ATTR_CONNECTION_POOL_IDLE_TIME => 30,
            ]
        );
        ok("$driver connect (pool)", ['max' => $poolMax, 'tasks' => $taskN]);
        ok("$driver attr server_version", $pdo->getAttribute(\PDO\PDO::ATTR_SERVER_VERSION));
        ok("$driver attr client_version", $pdo->getAttribute(\PDO\PDO::ATTR_CLIENT_VERSION));
        ok("$driver attr server_info", $pdo->getAttribute(\PDO\PDO::ATTR_SERVER_INFO));
        ok("$driver attr connection_status", $pdo->getAttribute(\PDO\PDO::ATTR_CONNECTION_STATUS));
    } catch (Throwable $e) {
        fail("$driver connect", $e);
    }

    try {
        $pdo->setAttribute(\PDO\PDO::ATTR_CASE, \PDO\PDO::CASE_UPPER);
        $stmt = $pdo->query('SELECT 1 AS foo');
        $row = $stmt ? $stmt->fetch(\PDO\PDO::FETCH_ASSOC) : false;
        ok("$driver ATTR_CASE=UPPER", $row);
        $pdo->setAttribute(\PDO\PDO::ATTR_CASE, \PDO\PDO::CASE_NATURAL);
    } catch (Throwable $e) {
        fail("$driver ATTR_CASE", $e);
    }

    try {
        $stmt = $pdo->prepare(
            'SELECT 1 AS v UNION ALL SELECT 2 AS v UNION ALL SELECT 3 AS v',
            [\PDO\PDO::ATTR_CURSOR => \PDO\PDO::CURSOR_SCROLL]
        );
        $stmt->execute();
        ok("$driver cursor scroll last", $stmt->fetch(\PDO\PDO::FETCH_ASSOC, \PDO\PDO::FETCH_ORI_LAST));
        ok("$driver cursor scroll first", $stmt->fetch(\PDO\PDO::FETCH_ASSOC, \PDO\PDO::FETCH_ORI_FIRST));
    } catch (Throwable $e) {
        fail("$driver cursor scroll", $e);
    }

    try {
        $stmt = $pdo->query('SELECT 1 AS grp, 10 AS val UNION ALL SELECT 1 AS grp, 20 AS val UNION ALL SELECT 2 AS grp, 30 AS val');
        $grouped = $stmt ? $stmt->fetchAll(\PDO\PDO::FETCH_GROUP | \PDO\PDO::FETCH_NUM) : [];
        ok("$driver fetchAll FETCH_GROUP|NUM", $grouped);

        $stmt = $pdo->query('SELECT 1 AS grp, 10 AS val UNION ALL SELECT 1 AS grp, 20 AS val UNION ALL SELECT 2 AS grp, 30 AS val');
        $unique = $stmt ? $stmt->fetchAll(\PDO\PDO::FETCH_UNIQUE | \PDO\PDO::FETCH_ASSOC) : [];
        ok("$driver fetchAll FETCH_UNIQUE|ASSOC", $unique);

        $stmt = $pdo->query('SELECT 1 AS grp, 10 AS val UNION ALL SELECT 2 AS grp, 30 AS val');
        $func = $stmt ? $stmt->fetchAll(\PDO\PDO::FETCH_FUNC, fn($grp, $val) => (int)$grp + (int)$val) : [];
        ok("$driver fetchAll FETCH_FUNC", $func);
    } catch (Throwable $e) {
        fail("$driver fetchAll flags", $e);
    }

    try {
        $stmt = $pdo->prepare('SELECT ? AS v');
        $stmt->bindValue(1, '123', \PDO\PDO::PARAM_INT);
        $stmt->execute();
        $v = $stmt->fetchColumn(0);
        ok("$driver bindValue PARAM_INT type", ['value' => $v, 'type' => gettype($v)]);
    } catch (Throwable $e) {
        fail("$driver bindValue types", $e);
    }

    // Use a normal table (NOT temporary) because a pool has multiple physical connections.
    try {
        if ($driver === 'mysql') {
            $pdo->exec('DROP TABLE IF EXISTS async_php_pdo_pool_t');
            $pdo->exec('CREATE TABLE async_php_pdo_pool_t (id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY, v INT NOT NULL) ENGINE=InnoDB');
        } else {
            $pdo->exec('DROP TABLE IF EXISTS async_php_pdo_pool_t');
            $pdo->exec('CREATE TABLE async_php_pdo_pool_t (id BIGSERIAL PRIMARY KEY, v INT NOT NULL)');
        }
        ok("$driver create table");
    } catch (Throwable $e) {
        fail("$driver create table", $e);
    }

    $ch = new Channel($taskN);

    // Main coroutine should not see a worker's transaction state.
    ok("$driver main inTransaction before", $pdo->inTransaction());

    for ($i = 0; $i < $taskN; $i++) {
        go(function () use ($pdo, $driver, $i, $ch) {
            try {
                if ($pdo->inTransaction()) {
                    throw new RuntimeException("unexpected inTransaction=true at task start");
                }

                // Make one task trigger an error, to ensure errorInfo is coroutine-local.
                $localErrorCode = null;
                if ($i === 0) {
                    try {
                        $pdo->query('SELECT * FROM __async_php_pdo_pool_not_exists__');
                    } catch (Throwable) {
                        $localErrorCode = $pdo->errorCode();
                    }
                }

                if (!$pdo->beginTransaction()) {
                    throw new RuntimeException("beginTransaction failed");
                }
                $pdo->exec('INSERT INTO async_php_pdo_pool_t(v) VALUES (' . (int)$i . ')');
                $id = $pdo->lastInsertId();
                if ($id === false || $id === '') {
                    throw new RuntimeException("lastInsertId failed");
                }
                if (!$pdo->rollBack()) {
                    throw new RuntimeException("rollBack failed");
                }

                $ch->push([
                    'ok' => true,
                    'i' => $i,
                    'id' => $id,
                    'errorCode' => $localErrorCode,
                ]);
            } catch (Throwable $e) {
                $ch->push([
                    'ok' => false,
                    'i' => $i,
                    'error' => $e->getMessage(),
                ]);
            }
        });
    }

    $okCount = 0;
    $failCount = 0;
    $errorCodeSeen = null;
    for ($n = 0; $n < $taskN; $n++) {
        [$v, $recvOk] = $ch->pop(10);
        if (!$recvOk) {
            throw new RuntimeException("channel pop timeout");
        }
        if (!is_array($v) || !array_key_exists('ok', $v)) {
            throw new RuntimeException("invalid channel payload");
        }
        if ($v['ok']) {
            $okCount++;
            if (($v['i'] ?? null) === 0) {
                $errorCodeSeen = $v['errorCode'] ?? null;
            }
        } else {
            $failCount++;
            echo "[TASK FAIL] " . json_encode($v, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE) . PHP_EOL;
        }
    }

    ok("$driver tasks ok", $okCount);
    ok("$driver tasks fail", $failCount);
    ok("$driver worker errorCode (task0)", $errorCodeSeen);

    // Verify main coroutine state was not polluted.
    ok("$driver main inTransaction after", $pdo->inTransaction());
    ok("$driver main errorCode after", $pdo->errorCode());

    // All inserts are rolled back; table should be empty.
    $stmt = $pdo->query('SELECT COUNT(*) AS c FROM async_php_pdo_pool_t');
    $count = $stmt ? $stmt->fetchColumn(0) : false;
    ok("$driver rows after rollbacks", $count);

    // Cleanup
    try {
        $pdo->exec('DROP TABLE IF EXISTS async_php_pdo_pool_t');
    } catch (Throwable) {
        // ignore
    }
});
