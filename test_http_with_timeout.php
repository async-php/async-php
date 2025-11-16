<?php

require 'vendor/autoload.php';

use Async\Kernel\Network\Http\HttpClient;

echo "Starting HTTP test with timeout...\n";

$fiber = new Fiber(function() {
    echo "Creating client with 10s timeout...\n";
    // timeout, connect_timeout, pool_idle_timeout, pool_max_idle, max_redirects
    $client = new HttpClient(10.0, 5.0, null, null, null);

    echo "Creating request...\n";
    $request = $client->get('https://httpbin.org/get');

    echo "Sending request...\n";
    $response = Fiber::suspend($request->send());

    echo "Got response, status: " . $response->status() . "\n";

    echo "Getting stream...\n";
    $stream = $response->stream();

    echo "Calling readAll()...\n";
    $body = Fiber::suspend($stream->readAll());

    echo "Got body, length: " . strlen($body) . "\n";
    echo "First 100 chars: " . substr($body, 0, 100) . "\n";
});

echo "Running fiber...\n";
\run($fiber);

echo "Done!\n";
