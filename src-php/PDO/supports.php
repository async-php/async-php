<?php

// PDO polyfill (requires native pdo extension to be disabled).
if (!class_exists('PDO', false)) {
    class_alias(PDO\PDO::class, 'PDO');
    class_alias(PDO\PDOStatement::class, 'PDOStatement');
    class_alias(PDO\PDOException::class, 'PDOException');

    class_alias(PDO\Pgsql::class, '\\Pdo\\Pgsql');
    class_alias(PDO\Mysql::class, '\\Pdo\\Mysql');
}

