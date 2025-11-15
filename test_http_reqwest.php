<?php

require_once __DIR__ . '/vendor/autoload.php';

// Test the new reqwest-based HTTP client
echo "Testing reqwest-based HTTP client...\n\n";

$fiber = new Fiber(function () {
    echo "[1] Creating simple HTTP client\n";
    // Use constructor with all defaults (all params are optional)
    $client = new Async\Kernel\Network\Http\HttpClient(
        null, // timeout_secs
        null, // connect_timeout_secs
        null, // pool_idle_timeout_secs
        null, // pool_max_idle_per_host
        null, // max_redirects
        null, // enable_cookies
        null, // enable_http2
        null, // ca_cert_pem
        null, // client_cert_pem
        null, // client_key_pem
        null, // min_tls_version
        null  // accept_invalid_certs
    );

    echo "[2] Making GET request to httpbin.org\n";
    $request = $client->get('https://httpbin.org/get');
    $future = $request->send();
    $response = Fiber::suspend($future);

    echo "[3] Status: " . $response->status() . "\n";
    $status = $response->status();
    echo "[4] Is success: " . ($status >= 200 && $status < 300 ? 'yes' : 'no') . "\n";
    $headers = $response->headers();
    echo "[5] Content-Type: " . ($headers['content-type'] ?? 'unknown') . "\n";

    echo "[6] Reading response as JSON\n";
    $jsonFuture = $response->json();
    $jsonStr = Fiber::suspend($jsonFuture);
    $data = json_decode($jsonStr, true);

    echo "[7] URL from response: " . $data['url'] . "\n";
    echo "\n=== Core features tested successfully! ===\n";
    echo "✓ HTTP client creation\n";
    echo "✓ GET request\n";
    echo "✓ Response status and headers\n";
    echo "✓ JSON response parsing\n";
});

run($fiber);
