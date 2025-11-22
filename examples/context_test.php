<?php

require_once __DIR__ . '/../vendor/autoload.php';

use Async\Context;
use Async\Kernel;

Kernel::run(function () {
    Context::set('user_id', 42);
    echo "[Main] user_id=" . Context::get('user_id') . "\n";

    Kernel::spawn(function () {
        echo "  [Child] user_id(default)=" . Context::get('user_id', 'none') . "\n";
        Context::set('user_id', 7);
        Kernel::sleep(10);
        echo "  [Child] user_id(after)=" . Context::get('user_id') . "\n";
    });

    Kernel::sleep(5);
    echo "[Main] user_id(still)=" . Context::get('user_id') . "\n";
    Kernel::sleep(30);
});

