<?php

require_once __DIR__ . '/../vendor/autoload.php';

use Async\Kernel;
use Async\Kernel\Network\Http\HttpClient;
use Async\Kernel\Network\Http\HttpRequest;

echo "=== HTTP Client Test ===\n";
echo "Testing HTTP/HTTPS client\n\n";

Kernel::run(function () {
    // Create HTTP client
    $client = new HttpClient();
    $client->setTimeout(30);

    // Test 1: HTTP request to baidu.com
    try {
        echo "Test 1: GET request to http://www.baidu.com\n";
        echo str_repeat('=', 70) . "\n";

        $request = new HttpRequest('GET', 'http://www.baidu.com');
        $request->setHeader('User-Agent', 'Mozilla/5.0 (compatible; async-php/1.0)');
        $request->setHeader('Accept', 'text/html');

        echo "Sending request...\n";
        $response = \Fiber::suspend($client->send($request));

        echo "Status: " . $response->getStatusCode() . " " . $response->getReasonPhrase() . "\n";
        echo "Version: HTTP/" . $response->getVersion() . "\n";

        $body = $response->getBody();
        if ($body !== null) {
            $content = \Fiber::suspend($body->readAll());
            echo "Body length: " . strlen($content) . " bytes\n";
        }

        echo "✓ Test 1 passed!\n\n";
    } catch (Exception $e) {
        echo "✗ Test 1 failed: " . $e->getMessage() . "\n\n";
    }

    // Test 2: HTTPS request to baidu.com
    try {
        echo "Test 2: GET request to https://www.baidu.com (HTTPS)\n";
        echo str_repeat('=', 70) . "\n";

        $request = new HttpRequest('GET', 'https://www.baidu.com');
        $request->setHeader('User-Agent', 'Mozilla/5.0 (compatible; async-php/1.0)');
        $request->setHeader('Accept', 'text/html');

        echo "Sending HTTPS request...\n";
        $response = \Fiber::suspend($client->send($request));

        echo "Status: " . $response->getStatusCode() . " " . $response->getReasonPhrase() . "\n";
        echo "Version: HTTP/" . $response->getVersion() . "\n";

        // Check for important headers
        $headers = $response->getHeaders();
        $contentType = $response->getHeader('content-type');
        $server = $response->getHeader('server');

        if ($contentType) {
            echo "Content-Type: " . $contentType . "\n";
        }
        if ($server) {
            echo "Server: " . $server . "\n";
        }

        $body = $response->getBody();
        if ($body !== null) {
            $content = \Fiber::suspend($body->readAll());
            echo "Body length: " . strlen($content) . " bytes\n";

            // Check if content looks like HTML
            if (strpos($content, '<!DOCTYPE') !== false || strpos($content, '<html') !== false) {
                echo "✓ Received HTML content\n";
            }
        }

        echo "✓ Test 2 passed!\n\n";
    } catch (Exception $e) {
        echo "✗ Test 2 failed: " . $e->getMessage() . "\n\n";
    }

    // Test 3: JSON API request
    try {
        echo "Test 3: GET request to api.github.com (JSON API)\n";
        echo str_repeat('=', 70) . "\n";

        $request = new HttpRequest('GET', 'https://api.github.com/');
        $request->setHeader('User-Agent', 'async-php/1.0');
        $request->setHeader('Accept', 'application/json');

        echo "Sending request...\n";
        $response = \Fiber::suspend($client->send($request));

        echo "Status: " . $response->getStatusCode() . " " . $response->getReasonPhrase() . "\n";
        echo "Version: HTTP/" . $response->getVersion() . "\n";

        $body = $response->getBody();
        if ($body !== null) {
            $content = \Fiber::suspend($body->readAll());
            echo "Body length: " . strlen($content) . " bytes\n";

            // Try to parse JSON
            $json = json_decode($content, true);
            if ($json && is_array($json)) {
                echo "✓ Valid JSON response\n";
                echo "API endpoints available: " . count($json) . "\n";
            }
        }

        echo "✓ Test 3 passed!\n\n";
    } catch (Exception $e) {
        echo "✗ Test 3 failed: " . $e->getMessage() . "\n\n";
    }

    echo str_repeat('=', 70) . "\n";
    echo "All tests completed!\n";
});
