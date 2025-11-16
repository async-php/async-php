<?php

require 'vendor/autoload.php';

use Async\Kernel\Network\Http\HttpClient;

echo "Testing HTTP to localhost (no TLS/DNS)...\n";

$fiber = new Fiber(function() {
    echo "Creating client...\n";
    $client = new HttpClient(30.0, 10.0, null, null, null);

    echo "Trying http://127.0.0.1:80 ...\n";
    $request = $client->get('http://127.0.0.1:80');

    echo "Sending request...\n";
    try {
        $response = Fiber::suspend($request->send());
        echo "Got response, status: " . $response->status() . "\n";
    } catch (Exception $e) {
        echo "Expected error (no server): " . $e->getMessage() . "\n";
    }

    echo "Done with local test\n";
});

echo "Running fiber...\n";
\run($fiber);

echo "Test completed!\n";
