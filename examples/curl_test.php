<?php
/**
 * Async cURL Example
 *
 * This example demonstrates async cURL operations using the cURL extension wrapper.
 * Makes real HTTP requests to test various cURL features.
 *
 * Usage:
 *   php -d extension=target/release/libasync_php.dylib examples/curl_test.php
 */

require_once __DIR__ . '/../vendor/autoload.php';

use Async\Kernel;
use Async\Time;
use Curl\Handle;

Kernel::run(function () {
    echo "=== Async cURL Example ===\n\n";

    // Test 1: Simple GET request
    echo "1. Simple GET request\n";
    $handle = curl_init('https://api.github.com/users/github');
    curl_setopt($handle, CURLOPT_RETURNTRANSFER, true);
    curl_setopt($handle, CURLOPT_USERAGENT, 'Async-PHP/1.0');
    $response = curl_exec($handle);

    if ($response === false) {
        echo "   Error: " . curl_error($handle) . "\n";
    } else {
        $data = json_decode($response, true);
        echo "   Username: " . $data['login'] . "\n";
        echo "   Name: " . $data['name'] . "\n";
        echo "   Public repos: " . $data['public_repos'] . "\n";
    }

    $info = curl_getinfo($handle);
    echo "   HTTP Status: " . $info['http_code'] . "\n";
    curl_close($handle);
    echo "\n";

    // Test 2: GET with custom headers
    echo "2. GET with custom headers\n";
    $handle = curl_init('https://httpbin.org/headers');
    curl_setopt($handle, CURLOPT_RETURNTRANSFER, true);
    curl_setopt($handle, CURLOPT_HTTPHEADER, [
        'X-Custom-Header: MyValue',
        'X-Request-ID: 12345',
        'User-Agent: Async-PHP/1.0'
    ]);
    $response = curl_exec($handle);

    if ($response !== false) {
        $data = json_decode($response, true);
        echo "   Custom headers received: " . count($data['headers']) . "\n";
        if (isset($data['headers']['X-Custom-Header'])) {
            echo "   X-Custom-Header: " . $data['headers']['X-Custom-Header'] . "\n";
        }
    }
    curl_close($handle);
    echo "\n";

    // Test 3: POST request with JSON
    echo "3. POST request with JSON data\n";
    $handle = curl_init('https://httpbin.org/post');
    $postData = json_encode(['name' => 'Alice', 'age' => 30, 'city' => 'New York']);
    curl_setopt_array($handle, [
        CURLOPT_RETURNTRANSFER => true,
        CURLOPT_CUSTOMREQUEST => 'POST',
        CURLOPT_POSTFIELDS => $postData,
        CURLOPT_HTTPHEADER => ['Content-Type: application/json'],
    ]);
    $response = curl_exec($handle);

    if ($response !== false) {
        $data = json_decode($response, true);
        echo "   POST data received: " . json_encode($data['json']) . "\n";
        echo "   Content-Type: " . $data['headers']['Content-Type'] . "\n";
    }
    curl_close($handle);
    echo "\n";

    // Test 4: POST with form data
    echo "4. POST with form data\n";
    $handle = curl_init('https://httpbin.org/post');
    $formData = 'field1=value1&field2=value2&field3=value3';
    curl_setopt_array($handle, [
        CURLOPT_RETURNTRANSFER => true,
        CURLOPT_POST => true,
        CURLOPT_POSTFIELDS => $formData,
    ]);
    $response = curl_exec($handle);

    if ($response !== false) {
        $data = json_decode($response, true);
        echo "   Form fields received: " . count($data['form']) . "\n";
        echo "   field1: " . $data['form']['field1'] . "\n";
    }
    curl_close($handle);
    echo "\n";

    // Test 5: Multiple concurrent requests with go()
    echo "5. Concurrent requests with go()\n";
    $results = [];

    go(function () use (&$results) {
        $handle = curl_init('https://jsonplaceholder.typicode.com/posts/1');
        curl_setopt($handle, CURLOPT_RETURNTRANSFER, true);
        $response = curl_exec($handle);
        if ($response !== false) {
            $data = json_decode($response, true);
            $results['post1'] = $data['title'];
        }
        curl_close($handle);
    });

    go(function () use (&$results) {
        $handle = curl_init('https://jsonplaceholder.typicode.com/posts/2');
        curl_setopt($handle, CURLOPT_RETURNTRANSFER, true);
        $response = curl_exec($handle);
        if ($response !== false) {
            $data = json_decode($response, true);
            $results['post2'] = $data['title'];
        }
        curl_close($handle);
    });

    go(function () use (&$results) {
        $handle = curl_init('https://jsonplaceholder.typicode.com/posts/3');
        curl_setopt($handle, CURLOPT_RETURNTRANSFER, true);
        $response = curl_exec($handle);
        if ($response !== false) {
            $data = json_decode($response, true);
            $results['post3'] = $data['title'];
        }
        curl_close($handle);
    });

    // Wait for concurrent tasks
    Time::sleep(5000);

    echo "   Concurrent request results:\n";
    foreach ($results as $key => $title) {
        echo "   $key: $title\n";
    }
    echo "\n";

    // Test 6: Redirect following
    echo "6. Following redirects\n";
    $handle = curl_init('https://httpbin.org/redirect/3');
    curl_setopt_array($handle, [
        CURLOPT_RETURNTRANSFER => true,
        CURLOPT_FOLLOWLOCATION => true,
        CURLOPT_MAXREDIRS => 10,
    ]);
    $response = curl_exec($handle);

    $info = curl_getinfo($handle);
    echo "   Final URL: " . $info['url'] . "\n";
    echo "   HTTP Status: " . $info['http_code'] . "\n";
    if ($response !== false) {
        $data = json_decode($response, true);
        // Use redirect_count from curl_getinfo instead of response data
        echo "   Redirects followed: " . $info['redirect_count'] . "\n";
    }
    curl_close($handle);
    echo "\n";

    // Test 7: Different HTTP methods
    echo "7. Various HTTP methods\n";

    // PUT request
    $handle = curl_init('https://httpbin.org/put');
    curl_setopt_array($handle, [
        CURLOPT_RETURNTRANSFER => true,
        CURLOPT_CUSTOMREQUEST => 'PUT',
        CURLOPT_POSTFIELDS => 'put_data=test',
    ]);
    $response = curl_exec($handle);
    if ($response !== false) {
        $data = json_decode($response, true);
        // Check if response contains expected URL to verify success
        echo "   PUT request status: " . (isset($data['url']) && strpos($data['url'], '/put') !== false ? 'PUT' : 'FAILED') . "\n";
    }
    curl_close($handle);

    // DELETE request
    $handle = curl_init('https://httpbin.org/delete');
    curl_setopt_array($handle, [
        CURLOPT_RETURNTRANSFER => true,
        CURLOPT_CUSTOMREQUEST => 'DELETE',
    ]);
    $response = curl_exec($handle);
    if ($response !== false) {
        $data = json_decode($response, true);
        // Check if response contains expected URL to verify success
        echo "   DELETE request status: " . (isset($data['url']) && strpos($data['url'], '/delete') !== false ? 'DELETE' : 'FAILED') . "\n";
    }
    curl_close($handle);

    // PATCH request
    $handle = curl_init('https://httpbin.org/patch');
    curl_setopt_array($handle, [
        CURLOPT_RETURNTRANSFER => true,
        CURLOPT_CUSTOMREQUEST => 'PATCH',
        CURLOPT_POSTFIELDS => json_encode(['patch' => 'data']),
        CURLOPT_HTTPHEADER => ['Content-Type: application/json'],
    ]);
    $response = curl_exec($handle);
    if ($response !== false) {
        $data = json_decode($response, true);
        // Check if response contains expected URL to verify success
        echo "   PATCH request status: " . (isset($data['url']) && strpos($data['url'], '/patch') !== false ? 'PATCH' : 'FAILED') . "\n";
    }
    curl_close($handle);
    echo "\n";

    // Test 8: Query parameters
    echo "8. Query parameters\n";
    $handle = curl_init('https://httpbin.org/get?param1=value1&param2=value2&param3=value3');
    curl_setopt($handle, CURLOPT_RETURNTRANSFER, true);
    $response = curl_exec($handle);

    if ($response !== false) {
        $data = json_decode($response, true);
        echo "   Query parameters received: " . count($data['args']) . "\n";
        echo "   param1: " . $data['args']['param1'] . "\n";
        echo "   param2: " . $data['args']['param2'] . "\n";
    }
    curl_close($handle);
    echo "\n";

    // Test 9: Response headers
    echo "9. Capturing response headers\n";
    $handle = curl_init('https://httpbin.org/response-headers?X-Response-Header=TestValue');
    curl_setopt_array($handle, [
        CURLOPT_RETURNTRANSFER => true,
        CURLOPT_HEADER => false,
    ]);
    $response = curl_exec($handle);

    $info = curl_getinfo($handle);
    echo "   Content-Type: " . $info['content_type'] . "\n";
    echo "   URL: " . $info['url'] . "\n";
    echo "   Total time: " . round($info['total_time'], 3) . "s\n";
    curl_close($handle);
    echo "\n";

    // Test 10: Error handling
    echo "10. Error handling\n";
    $handle = curl_init('https://invalid-domain-that-does-not-exist.example.com');
    curl_setopt($handle, CURLOPT_RETURNTRANSFER, true);
    curl_setopt($handle, CURLOPT_TIMEOUT, 3);
    $response = curl_exec($handle);

    if ($response === false) {
        $errno = curl_errno($handle);
        $error = curl_error($handle);
        echo "   Error number: $errno\n";
        echo "   Error message: $error\n";
        echo "   Error description: " . curl_strerror($errno) . "\n";
    }
    curl_close($handle);
    echo "\n";

    // Test 11: Connection pooling with multiple requests
    echo "11. Connection pooling (5 sequential requests)\n";
    $start = microtime(true);
    for ($i = 1; $i <= 5; $i++) {
        $handle = curl_init('https://httpbin.org/delay/0');
        curl_setopt($handle, CURLOPT_RETURNTRANSFER, true);
        $response = curl_exec($handle);
        curl_close($handle);
    }
    $elapsed = round((microtime(true) - $start) * 1000);
    echo "   5 requests completed in {$elapsed}ms\n";
    echo "\n";

    // Test 12: cURL info
    echo "12. cURL version and capabilities\n";
    $version = curl_version();
    echo "   Version: " . $version['version'] . "\n";
    echo "   SSL Version: " . $version['ssl_version'] . "\n";
    echo "   Protocols: " . implode(', ', array_slice($version['protocols'], 0, 5)) . "...\n";
    echo "\n";

    // Test 13: Reset handle
    echo "13. Reset handle\n";
    $handle = curl_init('https://httpbin.org/get');
    curl_setopt($handle, CURLOPT_RETURNTRANSFER, true);
    curl_setopt($handle, CURLOPT_CUSTOMREQUEST, 'POST');
    echo "   Before reset - custom request set\n";

    curl_reset($handle);
    echo "   After reset - options cleared\n";
    curl_close($handle);
    echo "\n";

    echo "=== All cURL tests completed ===\n";
});
