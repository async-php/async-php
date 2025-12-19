<?php

require_once __DIR__ . '/../../vendor/autoload.php';

use Async\Context;
use Async\Kernel;
use Async\Time;

Kernel::run(function () {
    Context::set('user_id', 42);
    echo "[Main] user_id=" . Context::get('user_id') . "\n";

    Kernel::spawn(function () {
        echo "  [Child] user_id(default)=" . Context::get('user_id', 'none') . "\n";
        Context::set('user_id', 7);
        Time::sleep(0.01);
        echo "  [Child] user_id(after)=" . Context::get('user_id') . "\n";
    });

    Time::sleep(0.005);
    echo "[Main] user_id(still)=" . Context::get('user_id') . "\n";
    Time::sleep(0.03);
});

