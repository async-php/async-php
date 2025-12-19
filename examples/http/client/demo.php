<?php

require_once __DIR__ . '/../../../vendor/autoload.php';

use Async\Kernel;
use Async\Network\Http\Client;

echo "=== HTTP Client Demo ===\n";
echo "Demonstrating the PHP-side HTTP Client API\n\n";

Kernel::run(function () {
    // Create a client instance with configuration
    $client = new Client([
        'timeout' => 30,
        'enable_cookies' => true
    ]);

    // Example 1: Simple GET request
    echo "1. Simple GET request:\n";
    echo str_repeat('-', 60) . "\n";

    $response = $client->get('https://api.github.com/', [
        'User-Agent' => 'Async-PHP-Demo/1.0'
    ]);
    
    echo "Status: {$response->getStatusCode()} {$response->getReasonPhrase()}\n";
    echo "Content-Type: {$response->getHeaderLine('Content-Type')}\n\n";

    // Example 2: GET with query parameters
    echo "2. GET with query parameters:\n";
    echo str_repeat('-', 60) . "\n";

    $query = http_build_query([
        'q' => 'language:php stars:>1000',
        'sort' => 'stars',
        'order' => 'desc',
        'per_page' => 5,
    ]);
    
    $response = $client->get('https://api.github.com/search/repositories?' . $query, [
        'User-Agent' => 'Async-PHP-Demo/1.0',
        'Accept' => 'application/vnd.github.v3+json'
    ]);

    $data = json_decode((string)$response->getBody(), true);
    if (isset($data['total_count'])) {
        echo "Found {$data['total_count']} repositories\n";
        echo "Top 5 PHP repositories:\n";
        foreach ($data['items'] as $i => $repo) {
            echo "  " . ($i + 1) . ". {$repo['full_name']} - {$repo['stargazers_count']} stars\n";
        }
    }
    echo "\n";

    // Example 3: POST with JSON body
    echo "3. POST with JSON data:\n";
    echo str_repeat('-', 60) . "\n";

    $body = [
        'username' => 'async-php-user',
        'email' => 'user@example.com',
        'active' => true,
    ];

    $response = $client->post('https://httpbin.org/post', $body);

    if ($response->getStatusCode() === 200) {
        $data = json_decode((string)$response->getBody(), true);
        echo "POST successful!\n";
        if (isset($data['json'])) {
            echo "Sent data: " . json_encode($data['json']) . "\n";
        }
    }
    echo "\n";

    // Example 4: Custom headers
    echo "4. Request with custom headers:\n";
    echo str_repeat('-', 60) . "\n";

    $response = $client->get('https://api.github.com/users/github', [
        'Accept' => 'application/vnd.github.v3+json',
        'User-Agent' => 'async-php-demo/1.0',
    ]);

    $user = json_decode((string)$response->getBody(), true);
    if (isset($user['login'])) {
        echo "User: {$user['login']}\n";
        echo "Name: {$user['name']}\n";
        echo "Public repos: {$user['public_repos']}\n\n";
    }

    // Example 6: Response helpers (using standard PSR-7)
    echo "6. Response helper methods:\n";
    echo str_repeat('-', 60) . "\n";

    $response = $client->get('https://www.baidu.com');
    $isSuccess = $response->getStatusCode() >= 200 && $response->getStatusCode() < 300;
    $contentType = $response->getHeaderLine('Content-Type');
    $isHtml = strpos($contentType, 'html') !== false;
    
    echo "Is successful (2xx): " . ($isSuccess ? 'Yes' : 'No') . "\n";
    echo "Is HTML: " . ($isHtml ? 'Yes' : 'No') . "\n";
    echo "Content length: " . ($response->getHeaderLine('Content-Length') ?: 'Unknown') . "\n";
    echo "Server: " . ($response->getHeaderLine('Server') ?: 'Unknown') . "\n\n";

    echo str_repeat('=', 60) . "\n";
    echo "Demo completed successfully!\n";
});