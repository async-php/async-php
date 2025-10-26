<?php

require_once __DIR__ . '/../vendor/autoload.php';

use Async\Kernel;
use Async\Time;
use Async\Network\UnixServer;

Kernel::run(function () {
    Kernel::enableCoroutine(Kernel::HOOK_UNIX);

    $sockPath = '/tmp/async_test.sock';
    if (file_exists($sockPath)) unlink($sockPath);

    // Server
    Kernel::spawn(function () use ($sockPath) {
        $server = new UnixServer($sockPath);
        echo "[Server] Listening on $sockPath...\n";
        while (true) {
            try {
                $socket = $server->accept();
                Kernel::spawn(function () use ($socket) {
                    $data = $socket->read();
                    $socket->write("Echo: " . $data);
                    $socket->close();
                });
            } catch (\Exception $e) {
                break;
            }
        }
    });

    Time::sleep(1000);

    echo "[Client] Connecting...\n";
    // stream_socket_client for Unix might not be hooked. Use fopen.
    $fp = fopen("unix://$sockPath", 'r+');
    
    if (!$fp) {
        echo "FAILURE: Could not connect.\n";
        return;
    }
    
    fwrite($fp, "Hello Unix");
    fflush($fp);
    $response = fread($fp, 1024);
    echo "[Client] Response: $response\n";
    
    fclose($fp);
    
    if ($response === "Echo: Hello Unix") {
        echo "SUCCESS: Unix Hook works.\n";
    } else {
        echo "FAILURE: Response mismatch.\n";
    }
    
    exit(0);
});

