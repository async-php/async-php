<?php
require_once __DIR__ . '/../../../vendor/autoload.php';
use Async\Kernel;
use Async\Network\Http\Client;

Kernel::run(function () {
    $client = new Client();
    
    echo "=== Test 1: text() with auto-decompression ===\n";
    $response = $client->get('http://www.baidu.com');
    
    echo "Content-Encoding: " . ($response->header('content-encoding') ?? 'none') . "\n";
    $body = $response->text();
    echo "Body length: " . strlen($body) . " bytes\n";
    echo "First 100 chars: " . substr($body, 0, 100) . "\n";
    echo "Is HTML? " . (strpos($body, '<!DOCTYPE') !== false ? "YES ✓" : "NO ✗") . "\n\n";
    
    echo "=== Test 2: json() with auto-decompression ===\n";
    $response2 = $client->get('https://api.github.com/');
    echo "Content-Encoding: " . ($response2->header('content-encoding') ?? 'none') . "\n";
    $data = $response2->json();
    echo "Is valid JSON? " . (is_array($data) ? "YES ✓" : "NO ✗") . "\n";
    if (is_array($data) && isset($data['current_user_url'])) {
        echo "Sample field: current_user_url = " . $data['current_user_url'] . "\n";
    }
});
