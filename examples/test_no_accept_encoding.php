<?php
require_once __DIR__ . '/../vendor/autoload.php';
use Async\Kernel;
use Async\Network\Http\Client;

Kernel::run(function () {
    $client = new Client();
    $client->setAutoDecompress(false); // 禁用 Accept-Encoding
    
    $response = $client->get('http://www.baidu.com');
    
    echo "Content-Encoding: " . ($response->getHeader('content-encoding') ?? 'none') . "\n";
    
    // Read first chunk
    $chunk = $response->readChunk(100);
    echo "First 100 bytes: " . substr($chunk, 0, 100) . "\n";
    echo "Looks like HTML? " . (strpos($chunk, '<') !== false ? "YES" : "NO") . "\n";
});
