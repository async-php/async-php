<?php

require_once __DIR__ . '/vendor/autoload.php';

$server = new \Async\Kernel\Network\Http\HttpServer();

echo "Available methods on HttpServer:\n";
$methods = get_class_methods($server);
sort($methods);
foreach ($methods as $method) {
    echo "  - $method\n";
}
