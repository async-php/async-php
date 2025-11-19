<?php

require 'vendor/autoload.php';

use Async\Kernel;

echo "=== Testing Simple HTTP POST ===\n\n";

Kernel::run(function () {
    $client = new \Async\Kernel\Network\Http\HttpClient();

    $jsonData = json_encode(['test' => 'data']);

    echo "Sending POST request...\n";
    $request = $client->post('https://httpbin.org/post');
    $request->bodyText($jsonData, 'application/json');

    $response = \Fiber::suspend($request->send());

    echo "Status: " . $response->status() . "\n";

    $body = \Fiber::suspend($response->text());
    $data = json_decode($body, true);

    if (isset($data['json'])) {
        echo "✓ Success: " . json_encode($data['json']) . "\n";
    } else {
        echo "✗ Failed\n";
    }
});
