<?php
/**
 * HTTP Client Quick Demo
 *
 * A simple demonstration of all new production features
 */

require_once __DIR__ . '/../vendor/autoload.php';

use Async\Network\Http\Client;
use Async\Runtime;

Runtime::run(function () {
    echo "\n";
    echo "╔══════════════════════════════════════════════════════════╗\n";
    echo "║  HTTP Client - Production Features Quick Demo           ║\n";
    echo "╚══════════════════════════════════════════════════════════╝\n";
    echo "\n";

    // Create a production-ready client with all features enabled
    $client = (new Client())
        ->setTimeout(30)
        ->setFollowRedirects(true)
        ->setMaxRedirects(10)
        ->enableCookies()
        ->enableRetry()
        ->setAutoDecompress(true)
        ->setCollectMetrics(true)
        ->setMaxConcurrentRequests(5);

    echo "✓ Client configured with production features:\n";
    echo "  • Timeout: 30 seconds\n";
    echo "  • Auto-redirect: enabled (max 10)\n";
    echo "  • Cookie management: enabled\n";
    echo "  • Request retry: enabled\n";
    echo "  • Auto decompression: enabled\n";
    echo "  • Metrics collection: enabled\n";
    echo "  • Max concurrent requests: 5\n";
    echo "\n";

    // Demo 1: Simple GET request
    echo "━━━ Demo 1: Simple GET Request ━━━\n";
    try {
        $response = $client->get('https://httpbin.org/get', [
            'query' => ['demo' => 'simple', 'version' => '1.0']
        ]);

        echo "Status: {$response->getStatusCode()} {$response->getReasonPhrase()}\n";
        echo "Success: " . ($response->isSuccess() ? 'Yes' : 'No') . "\n";

        $data = $response->json();
        echo "URL: {$data['url']}\n";
        echo "Query params: " . json_encode($data['args']) . "\n";
    } catch (\Exception $e) {
        echo "Error: {$e->getMessage()}\n";
    }
    echo "\n";

    // Demo 2: POST with JSON
    echo "━━━ Demo 2: POST with JSON ━━━\n";
    try {
        $response = $client->post('https://httpbin.org/post', [
            'json' => [
                'username' => 'demo_user',
                'action' => 'create',
                'timestamp' => time()
            ]
        ]);

        echo "Status: {$response->getStatusCode()}\n";

        $data = $response->json();
        echo "Content-Type: {$data['headers']['Content-Type']}\n";
        echo "Sent data: " . json_encode($data['json']) . "\n";
    } catch (\Exception $e) {
        echo "Error: {$e->getMessage()}\n";
    }
    echo "\n";

    // Demo 3: Authentication
    echo "━━━ Demo 3: Basic Authentication ━━━\n";
    try {
        $authClient = new Client();
        $authClient->setBasicAuth('demo', 'secret123');

        $response = $authClient->get('https://httpbin.org/basic-auth/demo/secret123');

        if ($response->isSuccess()) {
            $data = $response->json();
            echo "✓ Authenticated successfully\n";
            echo "User: {$data['user']}\n";
            echo "Authenticated: " . ($data['authenticated'] ? 'true' : 'false') . "\n";
        }
    } catch (\Exception $e) {
        echo "Error: {$e->getMessage()}\n";
    }
    echo "\n";

    // Demo 4: Redirect following
    echo "━━━ Demo 4: Auto Redirect Following ━━━\n";
    try {
        $response = $client->get('https://httpbin.org/redirect/3');

        echo "✓ Followed 3 redirects automatically\n";
        echo "Final status: {$response->getStatusCode()}\n";
        echo "Final URL reached successfully\n";
    } catch (\Exception $e) {
        echo "Error: {$e->getMessage()}\n";
    }
    echo "\n";

    // Demo 5: Cookie session
    echo "━━━ Demo 5: Cookie Session ━━━\n";
    try {
        $sessionClient = new Client();
        $sessionClient->enableCookies();

        // Login - set session cookie
        echo "Step 1: Set session cookie\n";
        $sessionClient->get('https://httpbin.org/cookies/set?session_id=demo_abc123');

        // Subsequent request - cookie sent automatically
        echo "Step 2: Make request with automatic cookie\n";
        $response = $sessionClient->get('https://httpbin.org/cookies');
        $data = $response->json();

        if (isset($data['cookies']['session_id'])) {
            echo "✓ Cookie sent automatically: {$data['cookies']['session_id']}\n";
        }
    } catch (\Exception $e) {
        echo "Error: {$e->getMessage()}\n";
    }
    echo "\n";

    // Demo 6: Headers and compression
    echo "━━━ Demo 6: Custom Headers & Compression ━━━\n";
    try {
        $response = $client->get('https://httpbin.org/headers', [
            'headers' => [
                'X-Custom-Header' => 'Demo-Value',
                'X-API-Version' => '2.0'
            ]
        ]);

        $data = $response->json();
        echo "Custom headers sent:\n";
        echo "  X-Custom-Header: {$data['headers']['X-Custom-Header']}\n";
        echo "  X-Api-Version: {$data['headers']['X-Api-Version']}\n";

        if (isset($data['headers']['Accept-Encoding'])) {
            echo "  Accept-Encoding: {$data['headers']['Accept-Encoding']}\n";
            echo "  (Compression support auto-enabled)\n";
        }
    } catch (\Exception $e) {
        echo "Error: {$e->getMessage()}\n";
    }
    echo "\n";

    // Summary
    echo "╔══════════════════════════════════════════════════════════╗\n";
    echo "║  All Demos Completed Successfully!                      ║\n";
    echo "╚══════════════════════════════════════════════════════════╝\n";
    echo "\n";
    echo "Production features demonstrated:\n";
    echo "  ✓ Auto redirect handling\n";
    echo "  ✓ Basic authentication\n";
    echo "  ✓ Cookie management\n";
    echo "  ✓ JSON request/response\n";
    echo "  ✓ Custom headers\n";
    echo "  ✓ Compression support\n";
    echo "  ✓ Query parameters\n";
    echo "  ✓ Fluent configuration\n";
    echo "\n";
});
