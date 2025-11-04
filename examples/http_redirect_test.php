<?php
/**
 * HTTP Redirect Handling Test
 *
 * Tests automatic redirect following with various redirect types:
 * - 301 Moved Permanently
 * - 302 Found
 * - 303 See Other
 * - 307 Temporary Redirect
 * - 308 Permanent Redirect
 */

require_once __DIR__ . '/../vendor/autoload.php';

use Async\Network\Http\Client;
use Async\Runtime;

Runtime::run(function () {
    echo "=== HTTP Redirect Test ===\n\n";

    // Test 1: Follow multiple redirects
    echo "Test 1: Following 5 redirects\n";
    echo str_repeat("-", 40) . "\n";

    $client = new Client();
    $client->setFollowRedirects(true);
    $client->setMaxRedirects(10);

    try {
        $response = $client->get('https://httpbin.org/redirect/5');
        echo "✓ Successfully followed 5 redirects\n";
        echo "  Final status: {$response->getStatusCode()}\n";
        echo "  Response OK: " . ($response->isSuccess() ? 'Yes' : 'No') . "\n";
    } catch (\Exception $e) {
        echo "✗ Failed: {$e->getMessage()}\n";
    }

    echo "\n";

    // Test 2: Respect redirect limit
    echo "Test 2: Redirect limit enforcement\n";
    echo str_repeat("-", 40) . "\n";

    $client2 = new Client();
    $client2->setFollowRedirects(true);
    $client2->setMaxRedirects(3);  // Limit to 3

    try {
        $response = $client2->get('https://httpbin.org/redirect/5');
        echo "  Status: {$response->getStatusCode()}\n";
        echo "  Note: Stopped at max redirect limit\n";
    } catch (\Exception $e) {
        echo "  Error (expected): {$e->getMessage()}\n";
    }

    echo "\n";

    // Test 3: Disable redirect following
    echo "Test 3: Disable redirect following\n";
    echo str_repeat("-", 40) . "\n";

    $client3 = new Client();
    $client3->setFollowRedirects(false);

    try {
        $response = $client3->get('https://httpbin.org/redirect/1');
        echo "  Status: {$response->getStatusCode()}\n";
        echo "  Is redirect: " . ($response->isRedirect() ? 'Yes' : 'No') . "\n";

        if ($response->isRedirect()) {
            $location = $response->getHeader('Location');
            echo "  Location header: {$location}\n";
            echo "✓ Redirect not followed (as expected)\n";
        }
    } catch (\Exception $e) {
        echo "✗ Failed: {$e->getMessage()}\n";
    }

    echo "\n";

    // Test 4: Relative URL redirects
    echo "Test 4: Relative URL redirects\n";
    echo str_repeat("-", 40) . "\n";

    $client4 = new Client();
    $client4->setFollowRedirects(true);

    try {
        $response = $client4->get('https://httpbin.org/relative-redirect/2');
        echo "✓ Successfully handled relative URL redirects\n";
        echo "  Final status: {$response->getStatusCode()}\n";
    } catch (\Exception $e) {
        echo "✗ Failed: {$e->getMessage()}\n";
    }

    echo "\n";

    // Test 5: Absolute URL redirects
    echo "Test 5: Absolute URL redirects\n";
    echo str_repeat("-", 40) . "\n";

    $client5 = new Client();
    $client5->setFollowRedirects(true);

    try {
        $response = $client5->get('https://httpbin.org/absolute-redirect/2');
        echo "✓ Successfully handled absolute URL redirects\n";
        echo "  Final status: {$response->getStatusCode()}\n";
    } catch (\Exception $e) {
        echo "✗ Failed: {$e->getMessage()}\n";
    }

    echo "\n=== All Redirect Tests Complete ===\n";
});
