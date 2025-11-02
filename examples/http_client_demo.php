<?php

require_once __DIR__ . '/../vendor/autoload.php';

use Async\Kernel;
use Async\Network\Http\Client;

echo "=== HTTP Client Demo ===\n";
echo "Demonstrating the PHP-side HTTP Client API\n\n";

Kernel::run(function () {
    // Create a client instance
    $client = new Client();
    $client->setTimeout(30);

    // Example 1: Simple GET request
    echo "1. Simple GET request:\n";
    echo str_repeat('-', 60) . "\n";

    $response = $client->get('https://api.github.com/');
    echo "Status: {$response->getStatusCode()} {$response->getReasonPhrase()}\n";
    echo "Content-Type: {$response->getContentType()}\n\n";

    // Example 2: GET with query parameters
    echo "2. GET with query parameters:\n";
    echo str_repeat('-', 60) . "\n";

    $response = $client->get('https://api.github.com/search/repositories', [
        'query' => [
            'q' => 'language:php stars:>1000',
            'sort' => 'stars',
            'order' => 'desc',
            'per_page' => 5,
        ],
    ]);

    $data = $response->json();
    echo "Found {$data['total_count']} repositories\n";
    echo "Top 5 PHP repositories:\n";
    foreach ($data['items'] as $i => $repo) {
        echo "  " . ($i + 1) . ". {$repo['full_name']} - {$repo['stargazers_count']} stars\n";
    }
    echo "\n";

    // Example 3: POST with JSON body
    echo "3. POST with JSON data:\n";
    echo str_repeat('-', 60) . "\n";

    $response = $client->post('https://httpbin.org/post', [
        'json' => [
            'username' => 'async-php-user',
            'email' => 'user@example.com',
            'active' => true,
        ],
    ]);

    if ($response->isSuccess()) {
        $data = $response->json();
        echo "POST successful!\n";
        echo "Sent data: " . json_encode($data['json']) . "\n";
    }
    echo "\n";

    // Example 4: Custom headers
    echo "4. Request with custom headers:\n";
    echo str_repeat('-', 60) . "\n";

    $response = $client->get('https://api.github.com/users/github', [
        'headers' => [
            'Accept' => 'application/vnd.github.v3+json',
            'User-Agent' => 'async-php-demo/1.0',
        ],
    ]);

    $user = $response->json();
    echo "User: {$user['login']}\n";
    echo "Name: {$user['name']}\n";
    echo "Public repos: {$user['public_repos']}\n\n";

    // Example 5: Using static quick methods
    echo "5. Quick static methods:\n";
    echo str_repeat('-', 60) . "\n";

    $response = Client::quickGet('https://api.github.com/');
    echo "Quick GET status: {$response->getStatusCode()}\n\n";

    // Example 6: Response helpers
    echo "6. Response helper methods:\n";
    echo str_repeat('-', 60) . "\n";

    $response = $client->get('https://www.baidu.com');
    echo "Is successful (2xx): " . ($response->isSuccess() ? 'Yes' : 'No') . "\n";
    echo "Is HTML: " . ($response->isHtml() ? 'Yes' : 'No') . "\n";
    echo "Content length: " . ($response->getContentLength() ?? 'Unknown') . "\n";
    echo "Server: " . ($response->getHeader('server') ?? 'Unknown') . "\n\n";

    // Example 7: Setting default headers
    echo "7. Client with default headers:\n";
    echo str_repeat('-', 60) . "\n";

    $client->setDefaultHeader('X-App-Version', '1.0.0');
    $client->setDefaultHeader('X-App-Name', 'async-php-demo');

    $response = $client->get('https://httpbin.org/headers');
    if ($response->isSuccess()) {
        $data = $response->json();
        echo "Custom headers sent:\n";
        echo "  X-App-Version: " . ($data['headers']['X-App-Version'] ?? 'Not found') . "\n";
        echo "  X-App-Name: " . ($data['headers']['X-App-Name'] ?? 'Not found') . "\n";
    }
    echo "\n";

    echo str_repeat('=', 60) . "\n";
    echo "Demo completed successfully!\n";
});
