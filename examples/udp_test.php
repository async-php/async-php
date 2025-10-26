<?php

require_once __DIR__ . '/../vendor/autoload.php';

use Async\Kernel;
use Async\Network\Udp\Socket;
use Async\Time;

Kernel::run(function () {
    Kernel::enableCoroutine(Kernel::HOOK_UDP);

    $port = 8082;
    
    // Start UDP Echo Server (Rust Native for reliability in test, wrapped in PHP)
    Kernel::spawn(function () use ($port) {
        $socket = Socket::bind("127.0.0.1:$port");
        echo "[Server] UDP Listening on $port...\n";
        while (true) {
            // Use native wrapper method for server side
            $res = $socket->recvFrom(1024);
            if ($res) {
                [$data, $peer] = $res;
                echo "[Server] Received from $peer: $data\n";
                $socket->sendTo("Echo: " . $data, $peer);
            }
        }
    });

    Time::sleep(100);

    echo "[Client] Sending UDP packet...\n";
    
    // stream_socket_client for UDP might not be hooked. Use fopen.
    $fp = fopen("udp://127.0.0.1:$port", 'r+');
    
    if (!$fp) {
        echo "FAILURE: Could not create UDP socket.\n";
        return;
    }
    
    fwrite($fp, "Hello UDP");
    fflush($fp);
    
    // Read response
    $response = fread($fp, 1024);
    echo "[Client] Response: $response\n";
    
    if ($response === "Echo: Hello UDP") {
        echo "SUCCESS: UDP Hook works.\n";
    } else {
        echo "FAILURE: Response mismatch.\n";
    }
    
    exit(0);
});