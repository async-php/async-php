<?php

// PDO polyfill (requires native pdo extension to be disabled).
if (!class_exists('PDO', false)) {
    class_alias(PDO\PDO::class, 'PDO');
    class_alias(PDO\PDOStatement::class, 'PDOStatement');
    class_alias(PDO\PDOException::class, 'PDOException');
}

// Driver specializations (PHP 8.4+ class names).
if (!class_exists('Pdo\\Pgsql', false)) {
    require_once __DIR__ . '/Driver/Pgsql.php';
}
if (!class_exists('Pdo\\Mysql', false)) {
    require_once __DIR__ . '/Driver/Mysql.php';
}
