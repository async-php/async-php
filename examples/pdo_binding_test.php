<?php

require __DIR__ . '/../vendor/autoload.php';

use Async\Kernel;

error_reporting(E_ALL);
ini_set('display_errors', '1');

function ok(string $label, mixed $value = null): void {
    echo "[OK] $label";
    if (func_num_args() > 1) {
        echo ": " . json_encode($value, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE) . PHP_EOL;
        return;
    }
    echo PHP_EOL;
}

function fail(string $label, Throwable $e): void {
    echo "[FAIL] $label: " . $e->getMessage() . PHP_EOL;
    throw $e;
}

Kernel::run(function () {
    // Test MySQL
    testDriver('mysql', [
        'dsn' => 'mysql:host=127.0.0.1;port=3306;dbname=mysql',
        'user' => 'root',
        'pass' => 'root',
    ]);

    // Test PostgreSQL
    testDriver('pgsql', [
        'dsn' => 'pgsql:host=127.0.0.1;port=5432;dbname=postgres',
        'user' => 'postgres',
        'pass' => 'postgres',
    ]);
});

function testDriver(string $driver, array $config): void {
    try {
        $pdo = new \PDO\PDO(
            $config['dsn'],
            $config['user'],
            $config['pass'],
            [
                \PDO\PDO::ATTR_ERRMODE => \PDO\PDO::ERRMODE_EXCEPTION,
                \PDO\PDO::ATTR_DEFAULT_FETCH_MODE => \PDO\PDO::FETCH_ASSOC,
            ]
        );
        ok("$driver connect");

        // Test 1: Single positional parameter
        $stmt = $pdo->prepare('SELECT ? AS value');
        $stmt->execute([42]);
        $result = $stmt->fetch();
        ok("$driver single positional", $result['value'] == 42);

        // Test 2: Multiple positional parameters
        $stmt = $pdo->prepare('SELECT ? AS a, ? AS b, ? AS c');
        $stmt->execute([1, 2, 3]);
        $result = $stmt->fetch();
        ok("$driver multiple positional", $result == ['a' => 1, 'b' => 2, 'c' => 3]);

        // Test 3: Single named parameter
        $stmt = $pdo->prepare('SELECT :value AS value');
        $stmt->execute([':value' => 'hello']);
        $result = $stmt->fetch();
        ok("$driver single named", $result['value'] === 'hello');

        // Test 4: Multiple named parameters
        $stmt = $pdo->prepare('SELECT :name AS name, :age AS age, :city AS city');
        $stmt->execute([':name' => 'Alice', ':age' => 30, ':city' => 'NYC']);
        $result = $stmt->fetch();
        ok("$driver multiple named", $result == ['name' => 'Alice', 'age' => 30, 'city' => 'NYC']);

        // Test 5: Mixed positional and named (positional first)
        $stmt = $pdo->prepare('SELECT ? AS a, :b AS b');
        $stmt->execute([10, ':b' => 20]);
        $result = $stmt->fetch();
        ok("$driver mixed pos+named", $result == ['a' => 10, 'b' => 20]);

        // Test 6: Placeholders in string literals should NOT be replaced
        $stmt = $pdo->prepare("SELECT '?' AS quoted, ':name' AS quoted_named");
        $stmt->execute();
        $result = $stmt->fetch();
        ok("$driver placeholders in strings", $result == ['quoted' => '?', 'quoted_named' => ':name']);

        // Test 7: Placeholders in comments should NOT be replaced
        $stmt = $pdo->prepare("SELECT 1 AS n -- comment with ? and :param\n");
        $stmt->execute();
        $result = $stmt->fetch();
        ok("$driver placeholders in line comments", $result['n'] == 1);

        $stmt = $pdo->prepare("SELECT 2 AS n /* block comment with ? and :param */");
        $stmt->execute();
        $result = $stmt->fetch();
        ok("$driver placeholders in block comments", $result['n'] == 2);

        // Test 8: PostgreSQL :: cast operator should NOT be treated as named param
        if ($driver === 'pgsql') {
            $stmt = $pdo->prepare('SELECT :val::integer AS casted');
            $stmt->execute([':val' => '42']);
            $result = $stmt->fetch();
            ok("$driver :: cast operator", $result['casted'] == 42);

            // Test 9: PostgreSQL dollar-quoted strings
            $stmt = $pdo->prepare("SELECT $$? and :name$$::text AS dollar_quoted");
            $stmt->execute();
            $result = $stmt->fetch();
            ok("$driver dollar-quoted strings", $result['dollar_quoted'] === '? and :name');

            // Test 10: PostgreSQL tagged dollar-quoted strings
            $stmt = $pdo->prepare("SELECT \$tag\$? and :name\$tag\$::text AS tagged");
            $stmt->execute();
            $result = $stmt->fetch();
            ok("$driver tagged dollar-quoted", $result['tagged'] === '? and :name');
        }

        // Test 11: E-strings (PostgreSQL)
        if ($driver === 'pgsql') {
            $stmt = $pdo->prepare("SELECT E'test\\n? and :name' AS estring");
            $stmt->execute();
            $result = $stmt->fetch();
            ok("$driver E-strings", strpos($result['estring'], '?') !== false);
        }

        // Test 12: Escaped quotes in strings
        $stmt = $pdo->prepare("SELECT '?' AS q1, 'it''s a ? test' AS q2");
        $stmt->execute();
        $result = $stmt->fetch();
        ok("$driver escaped quotes", $result['q1'] === '?' && strpos($result['q2'], 'it\'s') !== false);

        // Test 13: bindValue with different parameter types
        $stmt = $pdo->prepare('SELECT ? AS int_val, ? AS str_val');
        $stmt->bindValue(1, 123, \PDO\PDO::PARAM_INT);
        $stmt->bindValue(2, 'text', \PDO\PDO::PARAM_STR);
        $stmt->execute();
        $result = $stmt->fetch();
        ok("$driver bindValue types", $result == ['int_val' => 123, 'str_val' => 'text']);

        // Test 14: Named parameters without leading colon
        $stmt = $pdo->prepare('SELECT :id AS id, :name AS name');
        $stmt->execute(['id' => 99, 'name' => 'Bob']); // No leading : in keys
        $result = $stmt->fetch();
        ok("$driver named params without colon", $result == ['id' => 99, 'name' => 'Bob']);

        // Test 15: Complex query with multiple parameter types
        $sql = <<<SQL
SELECT
    ? AS pos1,           -- positional
    :named1 AS n1,       -- named
    '?' AS literal,      -- in string
    -- ? :comment_param  -- in comment
    ? AS pos2
FROM (SELECT 1) t
SQL;
        $stmt = $pdo->prepare($sql);
        $stmt->execute([10, ':named1' => 20, 30]);
        $result = $stmt->fetch();
        ok("$driver complex query", $result == ['pos1' => 10, 'n1' => 20, 'literal' => '?', 'pos2' => 30]);

        // Test 16: Special characters in string values
        $stmt = $pdo->prepare('SELECT ? AS val');
        $stmt->execute(["test'with\"quotes:and?marks"]);
        $result = $stmt->fetch();
        ok("$driver special chars in values", $result['val'] === "test'with\"quotes:and?marks");

    } catch (Throwable $e) {
        fail("$driver tests", $e);
    }
}
