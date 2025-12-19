<?php

require_once __DIR__ . '/../../vendor/autoload.php';

use Async\Kernel;
use Async\Time;
use Async\Network\Tcp\Server;

// This test requires running a server in the background or same process.
// We'll run a simple server in a fiber and connect to it.

Kernel::run(function () {
    Kernel::enableCoroutine(Kernel::HOOK_TCP);

    $port = 8081;
    
    // Start Server
    Kernel::spawn(function () use ($port) {
        $server = Server::bind("127.0.0.1:$port");
        echo "[Server] Listening on $port...\n";
        while (true) {
            try {
                $socket = $server->accept();
                Kernel::spawn(function () use ($socket) {
                    $data = $socket->read();
                    echo "[Server] Received (" . strlen($data) . "): " . $data . "\n";
                    if ($data) {
                        $socket->write("Echo: " . $data);
                    }
                    $socket->close();
                });
            } catch (\Exception $e) {
                break;
            }
        }
    });

    Time::sleep(0.1); // Give server time to start

    echo "[Client] Connecting...\n";
    // stream_socket_client is not hooked by stream_wrapper_register('tcp').
    // We use fopen which uses the registered wrapper.
    $fp = fopen("tcp://127.0.0.1:$port", 'r+');
    
    if (!$fp) {
        echo "FAILURE: Could not connect.\n";
        return;
    }
    
    // Wrapper stream_write
    fwrite($fp, "Hello TCP");
    fflush($fp);
    
    // Wrapper stream_read
    $response = fread($fp, 1024);
    echo "[Client] Response: $response\n";
    
    fclose($fp);
    
    if ($response === "Echo: Hello TCP") {
        echo "SUCCESS: TCP Hook works.\n";
    } else {
        echo "FAILURE: Response mismatch.\n";
    }
    
    // Exit after test
    exit(0);
});