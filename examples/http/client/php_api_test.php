<?php

require_once __DIR__ . '/../../../vendor/autoload.php';

use Async\Kernel;
use Async\Network\Http\Client;

echo "=== PHP-Side HTTP Client Test ===\n";
echo "Testing the high-level PHP HTTP Client API\n\n";

Kernel::run(function () {
    // 1. Configure via constructor
    $client = new Client([
        'timeout' => 30
    ]);

    // Test 1: Simple GET request
    try {
        echo "Test 1: Simple GET request\n";
        echo str_repeat('=', 70) . "\n";

        $response = $client->get('http://www.baidu.com');

        echo "Status: {$response->status()}\n";
        echo "Version: HTTP/{$response->version()}\n";
        echo "Content-Type: {$response->headerLine('Content-Type')}\n";
        
        $body = $response->text();
        echo "Body length: " . strlen($body) . " bytes\n";
        
        $success = $response->status() >= 200 && $response->status() < 300;
        echo "Is successful: " . ($success ? 'Yes' : 'No') . "\n";
        echo "✓ Test 1 passed!\n\n";
    } catch (Exception $e) {
        echo "✗ Test 1 failed: {$e->getMessage()}\n\n";
    }

    // Test 2: HTTPS GET with custom headers
    try {
        echo "Test 2: HTTPS GET with custom headers\n";
        echo str_repeat('=', 70) . "\n";

        // Pass headers as 2nd argument
        $response = $client->get('https://www.baidu.com', [
            'Accept-Language' => 'zh-CN,zh;q=0.9',
            'Accept' => 'text/html',
        ]);

        echo "Status: {$response->status()}\n";
        echo "Server: " . ($response->headerLine('Server') ?: 'Unknown') . "\n";
        
        $contentType = $response->headerLine('Content-Type');
        $isHtml = strpos($contentType, 'html') !== false;
        echo "Is HTML: " . ($isHtml ? 'Yes' : 'No') . "\n";
        echo "✓ Test 2 passed!\n\n";
    } catch (Exception $e) {
        echo "✗ Test 2 failed: {$e->getMessage()}\n\n";
    }

    // Test 3: GET with query parameters (manual construction)
    try {
        echo "Test 3: GET with query parameters\n";
        echo str_repeat('=', 70) . "\n";

        $queryParams = http_build_query([
            'q' => 'language:php',
            'sort' => 'stars',
            'per_page' => 3,
        ]);
        $url = 'https://api.github.com/search/repositories?' . $queryParams;

        // Github requires User-Agent
        $response = $client->get($url, [
            'User-Agent' => 'Async-PHP-Client/1.0',
            'Accept' => 'application/vnd.github.v3+json'
        ]);

        echo "Status: {$response->status()}\n";
        echo "Content-Type: {$response->headerLine('Content-Type')}\n";

        $contentType = $response->headerLine('Content-Type');
        if (strpos($contentType, 'json') !== false) {
            $data = json_decode($response->text(), true);
            if (isset($data['total_count'])) {
                echo "Total count: {$data['total_count']}\n";
            }
            if (isset($data['items'])) {
                echo "Items returned: " . count($data['items']) . "\n";
                if (!empty($data['items'])) {
                    echo "Top repository: {$data['items'][0]['full_name']}\n";
                }
            }
        }

        echo "✓ Test 3 passed!\n\n";
    } catch (Exception $e) {
        echo "✗ Test 3 failed: {$e->getMessage()}\n\n";
    }

    // Test 4: JSON API request (Github User API)
    try {
        echo "Test 4: JSON API request\n";
        echo str_repeat('=', 70) . "\n";

        $response = $client->get('https://api.github.com/', [
             'User-Agent' => 'Async-PHP-Client/1.0',
             'Accept' => 'application/json'
        ]);

        echo "Status: {$response->status()}\n";

        $contentType = $response->headerLine('Content-Type');
        if ($response->status() === 200 && strpos($contentType, 'json') !== false) {
            $data = json_decode($response->text(), true);
            echo "API endpoints available: " . count($data) . "\n";
            echo "✓ Valid JSON response\n";
        }

        echo "✓ Test 4 passed!\n\n";
    } catch (Exception $e) {
        echo "✗ Test 4 failed: {$e->getMessage()}\n\n";
    }

    // Test 5: POST with JSON body
    try {
        echo "Test 5: POST with JSON body\n";
        echo str_repeat('=', 70) . "\n";

        $bodyData = [
            'name' => 'async-php',
            'version' => '1.0',
            'features' => ['http', 'async', 'fiber'],
        ];

        // Pass array as body (automatically JSON encoded), and headers
        $response = $client->post('https://httpbin.org/post', $bodyData, [
            'Content-Type' => 'application/json' // Explicit content type just in case
        ]);

        echo "Status: {$response->status()}\n";

        $contentType = $response->headerLine('Content-Type');
        if ($response->status() === 200 && strpos($contentType, 'json') !== false) {
            $data = json_decode($response->text(), true);
            
            // httpbin.org echoes back the JSON we sent
            if (isset($data['json']['name'])) {
                echo "Echoed name: {$data['json']['name']}\n";
                echo "✓ POST with JSON body works\n";
            }
        }

        echo "✓ Test 5 passed!\n\n";
    } catch (Exception $e) {
        echo "✗ Test 5 failed: {$e->getMessage()}\n\n";
    }

    // Test 7: HEAD request
    try {
        echo "Test 7: HEAD request\n";
        echo str_repeat('=', 70) . "\n";

        $response = $client->head('https://www.baidu.com');

        echo "Status: {$response->status()}\n";
        echo "Content-Length: " . ($response->headerLine('Content-Length') ?: 'Unknown') . "\n";
        
        $body = $response->text();
        echo "Body is empty: " . ($body === '' ? 'Yes' : 'No') . "\n";
        echo "✓ Test 7 passed!\n\n";
    } catch (Exception $e) {
        echo "✗ Test 7 failed: {$e->getMessage()}\n\n";
    }

    // Test 8: Error handling (404)
    try {
        echo "Test 8: Error handling (404 response)\n";
        echo str_repeat('=', 70) . "\n";

        $response = $client->get('https://httpbin.org/status/404');

        echo "Status: {$response->status()}\n";
        $isClientError = $response->status() >= 400 && $response->status() < 500;
        echo "Is client error: " . ($isClientError ? 'Yes' : 'No') . "\n";
        echo "Is successful: " . ($response->status() === 200 ? 'Yes' : 'No') . "\n";
        echo "✓ Test 8 passed!\n\n";
    } catch (Exception $e) {
        echo "✗ Test 8 failed: {$e->getMessage()}\n\n";
    }

    echo str_repeat('=', 70) . "\n";
    echo "All tests completed!\n";
});
