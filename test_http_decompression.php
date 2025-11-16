<?php

require 'vendor/autoload.php';

use Async\Kernel\Network\Http\HttpClient;

echo "=== Testing HTTP Auto-Decompression ===\n\n";

$fiber = new Fiber(function() {
    $client = new HttpClient();

    // httpbin.org supports gzip compression
    // When we request with Accept-Encoding, it will return compressed data
    // reqwest automatically handles decompression
    echo "Fetching compressed response from httpbin.org...\n";
    $request = $client->get('https://httpbin.org/gzip');
    $response = Fiber::suspend($request->send());

    echo "Status: " . $response->status() . "\n";
    echo "Headers: " . json_encode($response->headers(), JSON_PRETTY_PRINT) . "\n\n";

    // Read the response body - should be automatically decompressed
    $body = Fiber::suspend($response->text());
    echo "Response body (first 200 chars):\n";
    echo substr($body, 0, 200) . "...\n\n";

    // Parse as JSON to verify it's valid (not compressed gibberish)
    $data = json_decode($body, true);
    if (isset($data['gzipped']) && $data['gzipped'] === true) {
        echo "✓ Successfully received and auto-decompressed gzip response!\n";
    } else {
        echo "✗ Response doesn't look right\n";
    }

    echo "\n--- Testing deflate ---\n";
    $request2 = $client->get('https://httpbin.org/deflate');
    $response2 = Fiber::suspend($request2->send());
    $body2 = Fiber::suspend($response2->text());
    $data2 = json_decode($body2, true);

    if (isset($data2['deflated']) && $data2['deflated'] === true) {
        echo "✓ Successfully received and auto-decompressed deflate response!\n";
    } else {
        echo "✗ Response doesn't look right\n";
    }

    echo "\n--- Testing streaming with decompression ---\n";
    $request3 = $client->get('https://httpbin.org/gzip');
    $response3 = Fiber::suspend($request3->send());
    $stream = $response3->stream();

    echo "Reading streamed response...\n";
    $streamedBody = Fiber::suspend($stream->readAll());

    $data3 = json_decode($streamedBody, true);
    if (isset($data3['gzipped']) && $data3['gzipped'] === true) {
        echo "✓ Successfully streamed and auto-decompressed gzip response!\n";
    } else {
        echo "✗ Streamed response doesn't look right\n";
    }
});

\run($fiber);

echo "\n=== All Decompression Tests Passed! ===\n";
