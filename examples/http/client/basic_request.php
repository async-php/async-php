<?php

require_once __DIR__ . '/../../../vendor/autoload.php';

use Async\Network\Http\Client;
use Async\Network\Http\Psr7Request;
use Async\Network\Http\Uri;
use Async\Kernel;

Kernel::run(function () {
    echo "=== HTTP Client Test ===\n";
    echo "Testing HTTP/HTTPS client\n\n";

    // Create client with common configurations
    $client = new Client([
        'timeout' => 5, // 5 seconds
        'max_redirects' => 5,
        'enable_cookies' => true,
    ]);

    // Test 1: HTTP request to baidu.com
    try {
        echo "Test 1: GET request to http://www.baidu.com\n";
        echo str_repeat('=', 70) . "\n";

        echo "Sending request...\n";
        $uri = new Uri('http://www.baidu.com');
        $request = (new Psr7Request('GET', $uri))
            ->withHeader('User-Agent', 'Mozilla/5.0 (compatible; async-php/1.0)')
            ->withHeader('Accept', 'text/html');
        $response = $client->sendRequest($request);

        echo "Status: " . $response->getStatusCode() . " " . $response->getReasonPhrase() . "\n";
        echo "Version: HTTP/" . $response->getProtocolVersion() . "\n";
        echo "Body length: " . strlen($response->getBody()->getContents()) . " bytes\n";

        echo "✓ Test 1 passed!\n\n";
    } catch (Exception $e) {
        echo "✗ Test 1 failed: " . $e->getMessage() . "\n\n";
    }

    // Test 2: HTTPS request to baidu.com
    try {
        echo "Test 2: GET request to https://www.baidu.com (HTTPS)\n";
        echo str_repeat('=', 70) . "\n";

        echo "Sending HTTPS request...\n";
        $uri = new Uri('https://www.baidu.com');
        $request = (new Psr7Request('GET', $uri))
            ->withHeader('User-Agent', 'Mozilla/5.0 (compatible; async-php/1.0)')
            ->withHeader('Accept', 'text/html');
        $response = $client->sendRequest($request);

        echo "Status: " . $response->getStatusCode() . " " . $response->getReasonPhrase() . "\n";
        echo "Version: HTTP/" . $response->getProtocolVersion() . "\n";

        // Check for important headers
        $contentType = $response->getHeaderLine('content-type');
        $server = $response->getHeaderLine('server');

        if ($contentType) {
            echo "Content-Type: " . $contentType . "\n";
        }
        if ($server) {
            echo "Server: " . $server . "\n";
        }

        $content = $response->getBody()->getContents();
        echo "Body length: " . strlen($content) . " bytes\n";

        // Check if content looks like HTML
        if (strpos($content, '<!DOCTYPE') !== false || strpos($content, '<html') !== false) {
            echo "✓ Received HTML content\n";
        }

        echo "✓ Test 2 passed!\n\n";
    } catch (Exception $e) {
        echo "✗ Test 2 failed: " . $e->getMessage() . "\n\n";
    }

    // Test 3: JSON API request
    try {
        echo "Test 3: GET request to api.github.com (JSON API)\n";
        echo str_repeat('=', 70) . "\n";

        echo "Sending request...\n";
        $uri = new Uri('https://api.github.com/');
        $request = (new Psr7Request('GET', $uri))
            ->withHeader('User-Agent', 'Mozilla/5.0 (compatible; async-php/1.0)')
            ->withHeader('Accept', 'application/json');
        $response = $client->sendRequest($request);

        echo "Status: " . $response->getStatusCode() . " " . $response->getReasonPhrase() . "\n";
        echo "Version: HTTP/" . $response->getProtocolVersion() . "\n";

        $content = $response->getBody()->getContents();
        echo "Body length: " . strlen($content) . " bytes\n";
        $json = json_decode($content, true);
        if ($json && is_array($json)) {
            echo "✓ Valid JSON response\n";
            echo "API endpoints available: " . count($json) . "\n";
        }

        echo "✓ Test 3 passed!\n\n";
    } catch (Exception $e) {
        echo "✗ Test 3 failed: " . $e->getMessage() . "\n\n";
    }

    echo str_repeat('=', 70) . "\n";
    echo "All tests completed!\n";
});
