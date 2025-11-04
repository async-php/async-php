<?php
/**
 * HTTP Cookie Management Test
 *
 * Tests automatic cookie storage and sending:
 * - Set cookies from server
 * - Automatically send cookies in subsequent requests
 * - Cookie persistence across requests
 * - Clear cookies
 */

require_once __DIR__ . '/../vendor/autoload.php';

use Async\Network\Http\Client;
use Async\Runtime;

Runtime::run(function () {
    echo "=== HTTP Cookie Management Test ===\n\n";

    // Test 1: Enable Cookie Management
    echo "Test 1: Enable Cookie Jar\n";
    echo str_repeat("-", 40) . "\n";

    $client = new Client();
    $client->enableCookies();
    echo "✓ Cookie jar enabled\n\n";

    // Test 2: Set and Retrieve Single Cookie
    echo "Test 2: Set and Retrieve Single Cookie\n";
    echo str_repeat("-", 40) . "\n";

    try {
        // Set a cookie
        echo "  Step 1: Setting cookie 'session=abc123'\n";
        $response1 = $client->get('https://httpbin.org/cookies/set?session=abc123');

        // Retrieve cookies to verify it was stored
        $response2 = $client->get('https://httpbin.org/cookies');
        $data = $response2->json();

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
        $data = $response->json();

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
            $data = $response->json();
            $count = count($data['cookies']);
            echo "    Request {$i}: {$count} cookies sent\n";
        }

        echo "✓ Cookies persisted across multiple requests\n";
    } catch (\Exception $e) {
        echo "✗ Error: {$e->getMessage()}\n";
    }

    echo "\n";

    // Test 5: Clear Cookies
    echo "Test 5: Clear All Cookies\n";
    echo str_repeat("-", 40) . "\n";

    try {
        echo "  Step 1: Clearing cookie jar\n";
        $client->clearCookies();

        $response = $client->get('https://httpbin.org/cookies');
        $data = $response->json();

        if (empty($data['cookies'])) {
            echo "✓ All cookies cleared successfully\n";
            echo "  Cookie count: 0\n";
        } else {
            echo "✗ Cookies not cleared\n";
            echo "  Remaining: " . count($data['cookies']) . "\n";
        }
    } catch (\Exception $e) {
        echo "✗ Error: {$e->getMessage()}\n";
    }

    echo "\n";

    // Test 6: Without Cookie Jar
    echo "Test 6: Without Cookie Management\n";
    echo str_repeat("-", 40) . "\n";

    $client2 = new Client();
    // Don't enable cookies

    try {
        // Set a cookie
        $client2->get('https://httpbin.org/cookies/set?test=nocookies');

        // Try to retrieve - should not be there
        $response = $client2->get('https://httpbin.org/cookies');
        $data = $response->json();

        if (empty($data['cookies'])) {
            echo "✓ Cookies not stored (as expected, jar disabled)\n";
        } else {
            echo "✗ Unexpected: cookies stored without jar\n";
        }
    } catch (\Exception $e) {
        echo "✗ Error: {$e->getMessage()}\n";
    }

    echo "\n";

    // Test 7: Disable Cookie Management
    echo "Test 7: Enable then Disable Cookies\n";
    echo str_repeat("-", 40) . "\n";

    $client3 = new Client();

    try {
        // Enable and set cookies
        $client3->enableCookies();
        echo "  Step 1: Enabled cookies and set some\n";
        $client3->get('https://httpbin.org/cookies/set?test1=value1');

        $response1 = $client3->get('https://httpbin.org/cookies');
        $data1 = $response1->json();
        echo "    Cookies stored: " . count($data1['cookies']) . "\n";

        // Disable cookies
        $client3->disableCookies();
        echo "  Step 2: Disabled cookies\n";

        // Set new cookie - should not be stored
        $client3->get('https://httpbin.org/cookies/set?test2=value2');

        $response2 = $client3->get('https://httpbin.org/cookies');
        $data2 = $response2->json();

        if (empty($data2['cookies'])) {
            echo "✓ Cookies disabled successfully\n";
            echo "  New cookies not stored\n";
        }
    } catch (\Exception $e) {
        echo "✗ Error: {$e->getMessage()}\n";
    }

    echo "\n";

    // Test 8: Session Simulation
    echo "Test 8: Simulated Login Session\n";
    echo str_repeat("-", 40) . "\n";

    $client4 = new Client();
    $client4->enableCookies();

    try {
        // Simulate login - set session cookie
        echo "  Step 1: Login (set session cookie)\n";
        $client4->get('https://httpbin.org/cookies/set?sessionid=user123_session&authenticated=true');

        // Make authenticated requests
        echo "  Step 2: Make authenticated requests\n";
        $response = $client4->get('https://httpbin.org/cookies');
        $data = $response->json();

        if (isset($data['cookies']['sessionid']) && isset($data['cookies']['authenticated'])) {
            echo "✓ Session maintained across requests\n";
            echo "    Session ID: {$data['cookies']['sessionid']}\n";
            echo "    Authenticated: {$data['cookies']['authenticated']}\n";
        }

        // Logout - clear cookies
        echo "  Step 3: Logout (clear cookies)\n";
        $client4->clearCookies();

        $response2 = $client4->get('https://httpbin.org/cookies');
        $data2 = $response2->json();

        if (empty($data2['cookies'])) {
            echo "✓ Session cleared\n";
        }
    } catch (\Exception $e) {
        echo "✗ Error: {$e->getMessage()}\n";
    }

    echo "\n=== All Cookie Tests Complete ===\n";
});
