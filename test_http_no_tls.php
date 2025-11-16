<?php

require 'vendor/autoload.php';

use Async\Kernel\Network\Http\HttpClient;

echo "Testing HTTP without TLS...\n";

$fiber = new Fiber(function() {
    echo "Creating client...\n";
    $client = new HttpClient();

    echo "Requesting http://example.com (HTTP, no TLS)...\n";
    $request = $client->get('http://example.com');

    echo "Sending request...\n";
    $response = Fiber::suspend($request->send());

    echo "Got response, status: " . $response->status() . "\n";
    $body = Fiber::suspend($response->text());
    echo "Body length: " . strlen($body) . "\n";
});

echo "Running fiber...\n";
\run($fiber);

echo "Done!\n";
