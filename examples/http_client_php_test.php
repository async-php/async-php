<?php

require_once __DIR__ . '/../vendor/autoload.php';

use Async\Kernel;
use Async\Network\Http\Client;

echo "=== PHP-Side HTTP Client Test ===\n";
echo "Testing the high-level PHP HTTP Client API\n\n";

Kernel::run(function () {
    $client = new Client();
    $client->setTimeout(30);

    // Test 1: Simple GET request
    try {
        echo "Test 1: Simple GET request\n";
        echo str_repeat('=', 70) . "\n";

        $response = $client->get('http://www.baidu.com');

        echo "Status: {$response->getStatusCode()} {$response->getReasonPhrase()}\n";
        echo "Version: HTTP/{$response->getVersion()}\n";
        echo "Content-Type: {$response->getContentType()}\n";
        echo "Body length: " . strlen($response->getBody()) . " bytes\n";
        echo "Is successful: " . ($response->isSuccess() ? 'Yes' : 'No') . "\n";
        echo "✓ Test 1 passed!\n\n";
    } catch (Exception $e) {
        echo "✗ Test 1 failed: {$e->getMessage()}\n\n";
    }

    // Test 2: HTTPS GET with custom headers
    try {
        echo "Test 2: HTTPS GET with custom headers\n";
        echo str_repeat('=', 70) . "\n";

        $response = $client->get('https://www.baidu.com', [
            'headers' => [
                'Accept-Language' => 'zh-CN,zh;q=0.9',
                'Accept' => 'text/html',
            ]
        ]);

        echo "Status: {$response->getStatusCode()}\n";
        echo "Server: " . ($response->getHeader('server') ?? 'Unknown') . "\n";
        echo "Is HTML: " . ($response->isHtml() ? 'Yes' : 'No') . "\n";
        echo "✓ Test 2 passed!\n\n";
    } catch (Exception $e) {
        echo "✗ Test 2 failed: {$e->getMessage()}\n\n";
    }

    // Test 3: GET with query parameters
    try {
        echo "Test 3: GET with query parameters\n";
        echo str_repeat('=', 70) . "\n";

        $response = $client->get('https://api.github.com/search/repositories', [
            'query' => [
                'q' => 'language:php',
                'sort' => 'stars',
                'per_page' => 3,
            ],
        ]);

        echo "Status: {$response->getStatusCode()}\n";
        echo "Content-Type: {$response->getContentType()}\n";

        if ($response->isJson()) {
            $data = $response->json();
            echo "Total count: {$data['total_count']}\n";
            echo "Items returned: " . count($data['items']) . "\n";

            if (!empty($data['items'])) {
                echo "Top repository: {$data['items'][0]['full_name']}\n";
            }
        }

        echo "✓ Test 3 passed!\n\n";
    } catch (Exception $e) {
        echo "✗ Test 3 failed: {$e->getMessage()}\n\n";
    }

    // Test 4: JSON API request
    try {
        echo "Test 4: JSON API request\n";
        echo str_repeat('=', 70) . "\n";

        $response = $client->get('https://api.github.com/', [
            'headers' => [
                'Accept' => 'application/json',
            ]
        ]);

        echo "Status: {$response->getStatusCode()}\n";

        if ($response->isSuccess() && $response->isJson()) {
            $data = $response->json();
            echo "API endpoints available: " . count($data) . "\n";
            echo "✓ Valid JSON response\n";
        }

        echo "✓ Test 4 passed!\n\n";
    } catch (Exception $e) {
        echo "✗ Test 4 failed: {$e->getMessage()}\n\n";
    }

    // Test 5: POST with JSON body (to a test endpoint)
    try {
        echo "Test 5: POST with JSON body\n";
        echo str_repeat('=', 70) . "\n";

        $response = $client->post('https://httpbin.org/post', [
            'json' => [
                'name' => 'async-php',
                'version' => '1.0',
                'features' => ['http', 'async', 'fiber'],
            ],
        ]);

        echo "Status: {$response->getStatusCode()}\n";

        if ($response->isSuccess() && $response->isJson()) {
            $data = $response->json();

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

    // Test 6: Quick static methods
    try {
        echo "Test 6: Quick static GET method\n";
        echo str_repeat('=', 70) . "\n";

        $response = Client::quickGet('https://api.github.com/');

        echo "Status: {$response->getStatusCode()}\n";
        echo "✓ Static method works\n";
        echo "✓ Test 6 passed!\n\n";
    } catch (Exception $e) {
        echo "✗ Test 6 failed: {$e->getMessage()}\n\n";
    }

    // Test 7: HEAD request
    try {
        echo "Test 7: HEAD request\n";
        echo str_repeat('=', 70) . "\n";

        $response = $client->head('https://www.baidu.com');

        echo "Status: {$response->getStatusCode()}\n";
        echo "Content-Length: " . ($response->getContentLength() ?? 'Unknown') . "\n";
        echo "Body is empty: " . (empty($response->getBody()) ? 'Yes' : 'No') . "\n";
        echo "✓ Test 7 passed!\n\n";
    } catch (Exception $e) {
        echo "✗ Test 7 failed: {$e->getMessage()}\n\n";
    }

    // Test 8: Error handling (404)
    try {
        echo "Test 8: Error handling (404 response)\n";
        echo str_repeat('=', 70) . "\n";

        $response = $client->get('https://httpbin.org/status/404');

        echo "Status: {$response->getStatusCode()}\n";
        echo "Is client error: " . ($response->isClientError() ? 'Yes' : 'No') . "\n";
        echo "Is successful: " . ($response->isSuccess() ? 'Yes' : 'No') . "\n";
        echo "✓ Test 8 passed!\n\n";
    } catch (Exception $e) {
        echo "✗ Test 8 failed: {$e->getMessage()}\n\n";
    }

    echo str_repeat('=', 70) . "\n";
    echo "All tests completed!\n";
});
