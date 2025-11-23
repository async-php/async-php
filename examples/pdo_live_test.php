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
    // MySQL
    try {
        $pdo = new \PDO\PDO(
            'mysql:host=127.0.0.1;port=3306;dbname=mysql',
            'root',
            'root',
            [
                \PDO\PDO::ATTR_ERRMODE => \PDO\PDO::ERRMODE_EXCEPTION,
                \PDO\PDO::ATTR_DEFAULT_FETCH_MODE => \PDO\PDO::FETCH_ASSOC,
            ]
        );
        ok('mysql connect');

        $stmt = $pdo->prepare('SELECT ? AS v, :n AS n');
        $stmt->execute([123, ':n' => 456]);
        ok('mysql placeholders', $stmt->fetch());

        $pdo->exec('DROP TEMPORARY TABLE IF EXISTS async_php_pdo_t');
        $pdo->exec('CREATE TEMPORARY TABLE async_php_pdo_t (id INT NOT NULL AUTO_INCREMENT PRIMARY KEY, val VARCHAR(50))');

        $stmt = $pdo->prepare('INSERT INTO async_php_pdo_t(val) VALUES (?)');
        $stmt->execute(['hello']);
        $id = $pdo->lastInsertId();
        ok('mysql lastInsertId', $id);

        $stmt = $pdo->prepare('SELECT id, val FROM async_php_pdo_t WHERE id = ?');
        $stmt->execute([(int)$id]);
        ok('mysql select row', $stmt->fetch());

        $pdo->beginTransaction();
        $pdo->exec("INSERT INTO async_php_pdo_t(val) VALUES ('in_tx')");
        $pdo->rollBack();
        $stmt = $pdo->prepare("SELECT COUNT(*) AS c FROM async_php_pdo_t WHERE val = 'in_tx'");
        $stmt->execute();
        ok('mysql rollback', $stmt->fetchColumn(0));
    } catch (Throwable $e) {
        fail('mysql', $e);
    }

    // PostgreSQL
    try {
        $pdo = new \PDO\PDO(
            'pgsql:host=127.0.0.1;port=5432;dbname=postgres',
            'postgres',
            'postgres',
            [
                \PDO\PDO::ATTR_ERRMODE => \PDO\PDO::ERRMODE_EXCEPTION,
                \PDO\PDO::ATTR_DEFAULT_FETCH_MODE => \PDO\PDO::FETCH_ASSOC,
            ]
        );
        ok('pgsql connect');

        $stmt = $pdo->prepare('SELECT :x::int + 1 AS y');
        $stmt->execute([':x' => 41]);
        ok('pgsql named + :: cast', $stmt->fetch());

        $pdo->exec('DROP TABLE IF EXISTS async_php_pdo_t');
        $pdo->exec('CREATE TEMP TABLE async_php_pdo_t (id SERIAL PRIMARY KEY, val TEXT)');

        $stmt = $pdo->prepare('INSERT INTO async_php_pdo_t(val) VALUES (:val)');
        $stmt->execute([':val' => 'hello']);
        $id = $pdo->lastInsertId();
        ok('pgsql lastInsertId', $id);

        $stmt = $pdo->prepare('INSERT INTO async_php_pdo_t(val) VALUES (?) RETURNING id');
        $stmt->execute(['returning']);
        ok('pgsql returning', $stmt->fetchColumn(0));

        $pdo->beginTransaction();
        $pdo->exec("INSERT INTO async_php_pdo_t(val) VALUES ('in_tx')");
        $pdo->rollBack();
        $stmt = $pdo->prepare("SELECT COUNT(*) FROM async_php_pdo_t WHERE val = 'in_tx'");
        $stmt->execute();
        ok('pgsql rollback', $stmt->fetchColumn(0));
    } catch (Throwable $e) {
        fail('pgsql', $e);
    }
});

