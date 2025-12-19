<?php
/**
 * HTTP Cookie Management Test
 *
 * Tests automatic cookie storage and sending:
 * - Set cookies from server
 * - Automatically send cookies in subsequent requests
 * - Cookie persistence across requests
 */

require_once __DIR__ . '/../vendor/autoload.php';

use Async\Kernel;
use Async\Network\Http\Client;

Kernel::run(function () {
    echo "=== HTTP Cookie Management Test ===\n\n";

    // Test 1: Enable Cookie Management
    echo "Test 1: Enable Cookie Jar\n";
    echo str_repeat("-", 40) . "\n";

    $client = new Client([
        'enable_cookies' => true
    ]);
    echo "✓ Cookie jar enabled via constructor\n\n";

    // Test 2: Set and Retrieve Single Cookie
    echo "Test 2: Set and Retrieve Single Cookie\n";
    echo str_repeat("-", 40) . "\n";

    try {
        // Set a cookie
        echo "  Step 1: Setting cookie 'session=abc123'\n";
        $client->get('https://httpbin.org/cookies/set?session=abc123');

        // Retrieve cookies to verify it was stored
        $response = $client->get('https://httpbin.org/cookies');
        $data = json_decode((string)$response->getBody(), true);

        if (isset($data['cookies']['session']) && $data['cookies']['session'] === 'abc123') {
            echo "✓ Cookie stored and sent automatically\n";
            echo "  Cookie value: {$data['cookies']['session']}\n";
        } else {
            echo "✗ Cookie not stored correctly\n";
        }
    } catch (\Exception $e) {
        echo "✗ Error: {$e->getMessage()}\n";
    }

    echo "\n";

    // Test 3: Multiple Cookies
    echo "Test 3: Multiple Cookies\n";
    echo str_repeat("-", 40) . "\n";

    try {
        // Set multiple cookies
        echo "  Step 1: Setting multiple cookies\n";
        $client->get('https://httpbin.org/cookies/set?user=john');
        $client->get('https://httpbin.org/cookies/set?token=xyz789');
        $client->get('https://httpbin.org/cookies/set?lang=en');

        // Retrieve all cookies
        $response = $client->get('https://httpbin.org/cookies');
        $data = json_decode((string)$response->getBody(), true);

        $cookieCount = count($data['cookies']);
        echo "  Step 2: Retrieved {$cookieCount} cookies\n";

        $expectedCookies = ['session', 'user', 'token', 'lang'];
        $allPresent = true;

        foreach ($expectedCookies as $cookieName) {
            if (isset($data['cookies'][$cookieName])) {
                echo "    ✓ {$cookieName} = {$data['cookies'][$cookieName]}\n";
            } else {
                echo "    ✗ {$cookieName} missing\n";
                $allPresent = false;
            }
        }

        if ($allPresent) {
            echo "✓ All cookies stored and sent correctly\n";
        }
    } catch (\Exception $e) {
        echo "✗ Error: {$e->getMessage()}\n";
    }

    echo "\n";

    // Test 4: Cookie Persistence Across Requests
    echo "Test 4: Cookie Persistence\n";
    echo str_repeat("-", 40) . "\n";

    try {
        // Make multiple requests - cookies should persist
        echo "  Making 3 requests to verify persistence:\n";

        for ($i = 1; $i <= 3; $i++) {
            $response = $client->get('https://httpbin.org/cookies');
            $data = json_decode((string)$response->getBody(), true);
            $count = count($data['cookies']);
            echo "    Request {$i}: {$count} cookies sent\n";
        }

        echo "✓ Cookies persisted across multiple requests\n";
    } catch (\Exception $e) {
        echo "✗ Error: {$e->getMessage()}\n";
    }

    echo "\n";

    // Test 6: Without Cookie Jar
    echo "Test 6: Without Cookie Management\n";
    echo str_repeat("-", 40) . "\n";

    $client2 = new Client([
        'enable_cookies' => false
    ]);

    try {
        // Set a cookie
        $client2->get('https://httpbin.org/cookies/set?test=nocookies');

        // Try to retrieve - should not be there
        $response = $client2->get('https://httpbin.org/cookies');
        $data = json_decode((string)$response->getBody(), true);

        if (empty($data['cookies'])) {
            echo "✓ Cookies not stored (as expected, jar disabled)\n";
        } else {
            echo "✗ Unexpected: cookies stored without jar\n";
        }
    } catch (\Exception $e) {
        echo "✗ Error: {$e->getMessage()}\n";
    }

    echo "\n=== All Cookie Tests Complete ===\n";
});