<?php

require_once __DIR__ . '/../../../vendor/autoload.php';

use Async\Kernel;
use Async\Network\Http\Client;

echo "=== HTTP Client with Custom Certificate ===\n";
echo "Testing custom CA certificate configuration\n\n";

Kernel::run(function () {
    $client = new Client();
    $client->setTimeout(30);

    // Example 1: Using system default certificates
    try {
        echo "Test 1: Using system default certificates\n";
        echo str_repeat('=', 70) . "\n";

        $response = $client->get('https://www.baidu.com');
        echo "Status: {$response->status()}\n";
        echo "✓ Test 1 passed with default certificates!\n\n";
    } catch (Exception $e) {
        echo "✗ Test 1 failed: {$e->getMessage()}\n\n";
    }

    // Example 2: Setting custom CA certificate
    // Note: This is just a demonstration. In real usage, you would load
    // your custom CA certificate from a file or configuration
    try {
        echo "Test 2: Setting custom CA certificate\n";
        echo str_repeat('=', 70) . "\n";

        // Example of loading a custom CA certificate from a file
        // $customCaCert = file_get_contents('/path/to/custom-ca.pem');
        // $client->setCaCert($customCaCert);

        echo "Custom CA certificate can be set using setCaCert() method\n";
        echo "Usage: \$client->setCaCert(\$pemString)\n";
        echo "✓ Test 2 explained!\n\n";
    } catch (Exception $e) {
        echo "✗ Test 2 failed: {$e->getMessage()}\n\n";
    }

    // Example 3: Setting client certificate for mutual TLS
    try {
        echo "Test 3: Setting client certificate for mTLS\n";
        echo str_repeat('=', 70) . "\n";

        // Example of loading client certificate and key
        // $clientCert = file_get_contents('/path/to/client-cert.pem');
        // $clientKey = file_get_contents('/path/to/client-key.pem');
        // $client->setClientCert($clientCert, $clientKey);

        echo "Client certificate can be set using setClientCert() method\n";
        echo "Usage: \$client->setClientCert(\$certPem, \$keyPem)\n";
        echo "This is useful for mutual TLS authentication\n";
        echo "✓ Test 3 explained!\n\n";
    } catch (Exception $e) {
        echo "✗ Test 3 failed: {$e->getMessage()}\n\n";
    }

    // Example 4: Clearing custom certificates
    try {
        echo "Test 4: Clearing custom certificates\n";
        echo str_repeat('=', 70) . "\n";

        // Clear custom CA certificate and revert to system defaults
        $client->clearCaCert();
        echo "Custom CA certificate cleared\n";

        // Clear client certificate
        $client->clearClientCert();
        echo "Client certificate cleared\n";

        // Test that we can still make requests with system defaults
        $response = $client->get('https://www.baidu.com');
        echo "Status: {$response->status()}\n";
        echo "✓ Test 4 passed! Back to using system certificates\n\n";
    } catch (Exception $e) {
        echo "✗ Test 4 failed: {$e->getMessage()}\n\n";
    }

    // Example 5: Practical example with self-signed certificate
    // (This would require an actual test server with self-signed cert)
    echo "Test 5: Practical use case\n";
    echo str_repeat('=', 70) . "\n";
    echo "In a real-world scenario, you might use custom certificates to:\n";
    echo "1. Connect to internal APIs with self-signed certificates\n";
    echo "2. Use corporate CA certificates for internal services\n";
    echo "3. Implement mutual TLS (mTLS) for enhanced security\n";
    echo "4. Test against local development servers with custom certs\n";
    echo "\n";
    echo "Example code:\n";
    echo "\$client = new Client();\n";
    echo "\$client->setCaCert(file_get_contents('ca.pem'));\n";
    echo "\$client->setClientCert(\n";
    echo "    file_get_contents('client.crt'),\n";
    echo "    file_get_contents('client.key')\n";
    echo ");\n";
    echo "\$response = \$client->get('https://internal-api.example.com/data');\n";
    echo "\n";

    echo str_repeat('=', 70) . "\n";
    echo "All tests completed!\n";
});
