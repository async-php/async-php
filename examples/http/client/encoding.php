<?php
require_once __DIR__ . '/../../../vendor/autoload.php';
use Async\Kernel;
use Async\Network\Http\Client;

Kernel::run(function () {
    $client = new Client();
    $response = $client->get('http://www.baidu.com');
    
    echo "Content-Encoding: " . ($response->getHeader('content-encoding') ?? 'none') . "\n";
    echo "Content-Type: " . ($response->getHeader('content-type') ?? 'none') . "\n";
    
    // Read first chunk to see if it's compressed
    $chunk = $response->readChunk(10);
    echo "First 10 bytes (hex): " . bin2hex($chunk) . "\n";
    echo "Is gzip? " . (substr($chunk, 0, 2) === "\x1f\x8b" ? "YES" : "NO") . "\n";
});
