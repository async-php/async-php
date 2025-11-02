<?php

require_once __DIR__ . '/../vendor/autoload.php';

use Async\Kernel;
use Async\Network\Tcp\Socket as TcpSocket;
use Async\Network\Tcp\Server as TcpServer;
use Async\Network\Unix\Socket as UnixSocket;

function test_net_errors() {
    echo ">> Testing Network Error Handling\n";

    // 1. TCP Connect Error (Connection refused)
    Kernel::spawn(function () {
        echo "[1] Testing TCP Connect Error (127.0.0.1:9999)...\n";
        try {
            // Ensure 9999 is not listening
            TcpSocket::connect("127.0.0.1:9999");
            echo "[1] FAILED: Should have thrown exception.\n";
        } catch (\Throwable $e) {
            echo "[1] SUCCESS: Caught expected exception: " . $e->getMessage() . "\n";
            // Verify it's not the generic PHP wrapper one if possible, but Rust's IO error
            // PHP wrapper says "Failed to connect to..."
            // Rust says "Connection refused..." (usually)
        }
    });

    // 2. TCP Bind Error (Permission denied or Address in use)
    Kernel::spawn(function () {
        // Bind to a privileged port (requires root, so should fail for normal user)
        // Or bind to an invalid address.
        $addr = "256.256.256.256:80"; // Invalid IP
        echo "[2] Testing TCP Bind Error ($addr)...\n";
        try {
            TcpServer::bind($addr);
            echo "[2] FAILED: Should have thrown exception.\n";
        } catch (\Throwable $e) {
            echo "[2] SUCCESS: Caught expected exception: " . $e->getMessage() . "\n";
        }
    });

    // 3. Unix Connect Error
    Kernel::spawn(function () {
        $path = "/tmp/non_existent_socket_12345.sock";
        echo "[3] Testing Unix Connect Error ($path)...\n";
        try {
            UnixSocket::connect($path);
            echo "[3] FAILED: Should have thrown exception.\n";
        } catch (\Throwable $e) {
            echo "[3] SUCCESS: Caught expected exception: " . $e->getMessage() . "\n";
        }
    });
}

Kernel::run(function () {
    test_net_errors();
    
    // Keep loop alive briefly to let async tasks run
    Kernel::sleep(500);
});
