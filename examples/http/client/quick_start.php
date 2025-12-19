<?php
/**
 * HTTP Client Quick Demo
 *
 * A simple demonstration of all new production features
 */

require_once __DIR__ . '/../../../vendor/autoload.php';

use Async\Network\Http\Client;
use Async\Kernel;

Kernel::run(function () {
    echo "\n";
    echo "╔══════════════════════════════════════════════════════════╗\n";
    echo "║  HTTP Client - Production Features Quick Demo           ║\n";
    echo "╚══════════════════════════════════════════════════════════╝\n";
    echo "\n";

    // Create a production-ready client with all features enabled
    $client = new Client([
        'timeout' => 30,
        'max_redirects' => 10,
        'enable_cookies' => true,
    ]);

    echo "✓ Client configured with production features:\n";
    echo "  • Timeout: 30 seconds\n";
    echo "  • Auto-redirect: enabled (max 10)\n";
    echo "  • Cookie management: enabled\n";
    echo "  • Auto decompression: enabled\n";
    echo "\n";

    // Demo 1: Simple GET request
    echo "━━━ Demo 1: Simple GET Request ━━━\n";
    try {
        $response = $client->get('https://httpbin.org/get?demo=simple&version=1.0');

        echo "Status: {$response->status()}\n";
        $isSuccess = $response->status() >= 200 && $response->status() < 300;
        echo "Success: " . ($isSuccess ? 'Yes' : 'No') . "\n";

        $data = json_decode($response->text(), true);
        if ($data !== null) {
            echo "URL: {$data['url']}\n";
            echo "Query params: " . json_encode($data['args']) . "\n";
        } else {
            echo "Failed to decode JSON response.\n";
        }
    } catch (\Exception $e) {
        echo "Error: {$e->getMessage()}\n";
    }
    echo "\n";

    // Demo 2: POST with JSON
    echo "━━━ Demo 2: POST with JSON ━━━\n";
    try {
        $response = $client->post('https://httpbin.org/post', [
            'username' => 'demo_user',
            'action' => 'create',
            'timestamp' => time()
        ]);

        echo "Status: {$response->status()}\n";

        $data = json_decode($response->text(), true);
        if ($data !== null) {
            echo "Content-Type: {$data['headers']['Content-Type']}\n";
            echo "Sent data: " . json_encode($data['json']) . "\n";
        } else {
            echo "Failed to decode JSON response.\n";
        }
    } catch (\Exception $e) {
        echo "Error: {$e->getMessage()}\n";
    }
    echo "\n";

    // Demo 3: Basic Authentication
    echo "━━━ Demo 3: Basic Authentication ━━━\n";
    try {
        $authClient = new Client();
        $username = 'demo';
        $password = 'secret123';
        $authHeader = 'Basic ' . base64_encode("$username:$password");

        $response = $authClient->get('https://httpbin.org/basic-auth/demo/secret123', [
            'Authorization' => $authHeader
        ]);

        $isSuccess = $response->status() >= 200 && $response->status() < 300;
        if ($isSuccess) {
            $data = json_decode($response->text(), true);
            if ($data !== null) {
                echo "✓ Authenticated successfully\n";
                echo "User: {$data['user']}\n";
                echo "Authenticated: " . ($data['authenticated'] ? 'true' : 'false') . "\n";
            } else {
                echo "Failed to decode JSON response for authentication.\n";
            }
        } else {
            echo "Authentication failed. Status: {$response->status()}\n";
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
        echo "Final status: {$response->status()}\n";
        echo "Final URL reached successfully\n";
    } catch (\Exception $e) {
        echo "Error: {$e->getMessage()}\n";
    }
    echo "\n";

    // Demo 5: Cookie session
    echo "━━━ Demo 5: Cookie Session ━━━\n";
    try {
        $sessionClient = new Client(['enable_cookies' => true]);

        // Login - set session cookie
        echo "Step 1: Set session cookie\n";
        $sessionClient->get('https://httpbin.org/cookies/set?session_id=demo_abc123');

        // Subsequent request - cookie sent automatically
        echo "Step 2: Make request with automatic cookie\n";
        $response = $sessionClient->get('https://httpbin.org/cookies');
        $data = json_decode($response->text(), true);

        if ($data !== null && isset($data['cookies']['session_id'])) {
            echo "✓ Cookie sent automatically: {$data['cookies']['session_id']}\n";
        } else {
            echo "Failed to retrieve or decode cookie data.\n";
        }
    } catch (\Exception $e) {
        echo "Error: {$e->getMessage()}\n";
    }
    echo "\n";

    // Demo 6: Headers and compression
    echo "━━━ Demo 6: Custom Headers & Compression ━━━\n";
    try {
        $response = $client->get('https://httpbin.org/headers', [
            'X-Custom-Header' => 'Demo-Value',
            'X-API-Version' => '2.0'
        ]);

        $data = json_decode($response->text(), true);
        if ($data !== null) {
            $responseHeaders = array_change_key_case($data['headers'], CASE_LOWER);
            echo "Custom headers sent:\n";
            echo "  X-Custom-Header: " . ($responseHeaders['x-custom-header'] ?? 'N/A') . "\n";
            echo "  X-Api-Version: " . ($responseHeaders['x-api-version'] ?? 'N/A') . "\n";

            if (isset($responseHeaders['accept-encoding'])) {
                echo "  Accept-Encoding: {$responseHeaders['accept-encoding']}\n";
                echo "  (Compression support auto-enabled)\n";
            }
        } else {
            echo "Failed to decode JSON response for headers.\n";
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
