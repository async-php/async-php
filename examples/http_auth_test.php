<?php
/**
 * HTTP Authentication Test
 *
 * Tests different authentication methods:
 * - Basic Authentication
 * - Bearer Token
 * - Authentication clearing
 */

require_once __DIR__ . '/../vendor/autoload.php';

use Async\Network\Http\Client;
use Async\Runtime;

Runtime::run(function () {
    echo "=== HTTP Authentication Test ===\n\n";

    // Test 1: Basic Authentication - Success
    echo "Test 1: Basic Authentication (Success)\n";
    echo str_repeat("-", 40) . "\n";

    $client = new Client();
    $client->setBasicAuth('user', 'passwd');

    try {
        $response = $client->get('https://httpbin.org/basic-auth/user/passwd');
        $data = $response->json();

        if ($response->isSuccess()) {
            echo "✓ Authentication successful\n";
            echo "  Authenticated: " . ($data['authenticated'] ? 'true' : 'false') . "\n";
            echo "  User: {$data['user']}\n";
        } else {
            echo "✗ Authentication failed\n";
            echo "  Status: {$response->getStatusCode()}\n";
        }
    } catch (\Exception $e) {
        echo "✗ Error: {$e->getMessage()}\n";
    }

    echo "\n";

    // Test 2: Basic Authentication - Failure
    echo "Test 2: Basic Authentication (Wrong Password)\n";
    echo str_repeat("-", 40) . "\n";

    $client2 = new Client();
    $client2->setBasicAuth('user', 'wrong_password');

    try {
        $response = $client2->get('https://httpbin.org/basic-auth/user/passwd');

        if ($response->getStatusCode() === 401) {
            echo "✓ Correctly rejected with 401 Unauthorized\n";
            echo "  Status: {$response->getStatusCode()}\n";
        } else {
            echo "✗ Unexpected status: {$response->getStatusCode()}\n";
        }
    } catch (\Exception $e) {
        echo "  Exception (expected): {$e->getMessage()}\n";
    }

    echo "\n";

    // Test 3: Bearer Token Authentication
    echo "Test 3: Bearer Token Authentication\n";
    echo str_repeat("-", 40) . "\n";

    $client3 = new Client();
    $token = 'my-secret-token-123456';
    $client3->setBearerToken($token);

    try {
        $response = $client3->get('https://httpbin.org/bearer');
        $data = $response->json();

        if ($response->isSuccess() && $data['authenticated']) {
            echo "✓ Bearer token authentication successful\n";
            echo "  Token received: {$data['token']}\n";
            echo "  Authenticated: true\n";
        } else {
            echo "✗ Bearer token authentication failed\n";
        }
    } catch (\Exception $e) {
        echo "✗ Error: {$e->getMessage()}\n";
    }

    echo "\n";

    // Test 4: Clear Authentication
    echo "Test 4: Clear Authentication\n";
    echo str_repeat("-", 40) . "\n";

    $client4 = new Client();
    $client4->setBasicAuth('user', 'passwd');
    echo "  Step 1: Set Basic Auth\n";

    // Clear it
    $client4->clearAuth();
    echo "  Step 2: Clear Auth\n";

    try {
        $response = $client4->get('https://httpbin.org/basic-auth/user/passwd');

        if ($response->getStatusCode() === 401) {
            echo "✓ Authentication cleared successfully\n";
            echo "  Status: 401 Unauthorized (as expected)\n";
        } else {
            echo "✗ Unexpected status: {$response->getStatusCode()}\n";
        }
    } catch (\Exception $e) {
        echo "  Error (expected): " . substr($e->getMessage(), 0, 50) . "...\n";
    }

    echo "\n";

    // Test 5: Switch Authentication Methods
    echo "Test 5: Switch Between Auth Methods\n";
    echo str_repeat("-", 40) . "\n";

    $client5 = new Client();

    // First use Basic Auth
    $client5->setBasicAuth('user', 'passwd');
    echo "  Step 1: Using Basic Auth\n";

    try {
        $response1 = $client5->get('https://httpbin.org/basic-auth/user/passwd');
        if ($response1->isSuccess()) {
            echo "    ✓ Basic Auth works\n";
        }
    } catch (\Exception $e) {
        echo "    ✗ Basic Auth failed\n";
    }

    // Switch to Bearer Token
    $client5->setBearerToken('new-token-xyz');
    echo "  Step 2: Switched to Bearer Token\n";

    try {
        $response2 = $client5->get('https://httpbin.org/bearer');
        $data = $response2->json();

        if ($response2->isSuccess() && $data['token'] === 'new-token-xyz') {
            echo "    ✓ Bearer Token works\n";
            echo "    Token: {$data['token']}\n";
        }
    } catch (\Exception $e) {
        echo "    ✗ Bearer Token failed\n";
    }

    echo "\n";

    // Test 6: Headers Inspection
    echo "Test 6: Authentication Headers Inspection\n";
    echo str_repeat("-", 40) . "\n";

    $client6 = new Client();
    $client6->setBasicAuth('testuser', 'testpass');

    try {
        // httpbin.org/headers returns all request headers
        $response = $client6->get('https://httpbin.org/headers');
        $data = $response->json();

        if (isset($data['headers']['Authorization'])) {
            echo "✓ Authorization header present\n";
            echo "  Header: " . substr($data['headers']['Authorization'], 0, 20) . "...\n";
            echo "  (Base64 encoded credentials)\n";
        } else {
            echo "✗ Authorization header missing\n";
        }
    } catch (\Exception $e) {
        echo "✗ Error: {$e->getMessage()}\n";
    }

    echo "\n=== All Authentication Tests Complete ===\n";
});
