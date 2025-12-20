<?php
/**
 * HTTP Authentication Test
 *
 * Tests different authentication methods:
 * - Basic Authentication
 * - Bearer Token
 */

require_once __DIR__ . '/../../../vendor/autoload.php';

use Async\Kernel;
use Async\Network\Http\Client;

Kernel::run(function () {
    echo "=== HTTP Authentication Test ===\n\n";

    // Test 1: Basic Authentication - Success
    echo "Test 1: Basic Authentication (Success)\n";
    echo str_repeat("-", 40) . "\n";

    $client = new Client();
    
    // Manually set Basic Auth header
    $authHeader = 'Basic ' . base64_encode('user:passwd');

    try {
        $response = $client->get('https://httpbin.org/basic-auth/user/passwd', [
            'Authorization' => $authHeader
        ]);
        
        $body = $response->text();
        $data = json_decode($body, true);
        $statusCode = $response->getStatusCode();

        if ($statusCode === 200) {
            echo "✓ Authentication successful\n";
            echo "  Authenticated: " . ($data['authenticated'] ? 'true' : 'false') . "\n";
            echo "  User: {$data['user']}\n";
        } else {
            echo "✗ Authentication failed\n";
            echo "  Status: {$statusCode}\n";
        }
    } catch (\Exception $e) {
        echo "✗ Error: {$e->getMessage()}\n";
    }

    echo "\n";

    // Test 2: Basic Authentication - Failure
    echo "Test 2: Basic Authentication (Wrong Password)\n";
    echo str_repeat("-", 40) . "\n";

    $authHeaderWrong = 'Basic ' . base64_encode('user:wrong_password');

    try {
        $response = $client->get('https://httpbin.org/basic-auth/user/passwd', [
            'Authorization' => $authHeaderWrong
        ]);

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

    $token = 'my-secret-token-123456';
    
    try {
        $response = $client->get('https://httpbin.org/bearer', [
            'Authorization' => "Bearer $token"
        ]);
        
        $body = $response->text();
        $data = json_decode($body, true);

        if ($response->getStatusCode() === 200 && $data['authenticated']) {
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

    // Test 6: Headers Inspection
    echo "Test 6: Authentication Headers Inspection\n";
    echo str_repeat("-", 40) . "\n";

    try {
        // httpbin.org/headers returns all request headers
        $response = $client->get('https://httpbin.org/headers', [
            'Authorization' => 'Basic ' . base64_encode('testuser:testpass')
        ]);
        
        $body = $response->text();
        $data = json_decode($body, true);

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
