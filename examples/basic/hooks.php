<?php

require_once __DIR__ . '/../../vendor/autoload.php';

use Async\Kernel;
use Async\Time;

Kernel::run(function () {
    echo "[Test] Enabling Coroutine Hooks...\n";
    Kernel::enableCoroutine(Kernel::HOOK_FILE);
    // Note: HOOK_TCP via wrapper only affects fopen('tcp://...'), not stream_socket_client.
    
    // --- Test 1: File Hook ---
    $tmpFile = __DIR__ . '/test_hook_file.txt';
    if (file_exists($tmpFile)) unlink($tmpFile);
    
    echo "[Test] Writing file (hooked)...\n";
    file_put_contents($tmpFile, "Hello Async FS!");
    
    echo "[Test] Reading file (hooked)...\n";
    $content = file_get_contents($tmpFile);
    echo "[Test] Content: $content\n";
    
    if ($content === "Hello Async FS!") {
        echo "[Test] File Hook PASSED.\n";
    } else {
        echo "[Test] File Hook FAILED.\n";
    }
    
    if (file_exists($tmpFile)) unlink($tmpFile);
    
    // --- Test 2: TCP Hook (fopen) ---
    // We can only hook fopen('tcp://...') via wrapper.
    // stream_socket_client uses transport which we can't override easily in PHP.
    
    echo "[Test] Starting Mock Server for fopen(tcp://)...";
    $server = stream_socket_server("tcp://127.0.0.1:8083", $errno, $errstr);
    if (!$server) {
        echo "Failed to start server: $errstr\n";
        return;
    }
    
    // Enable TCP hook now
    Kernel::enableCoroutine(Kernel::HOOK_TCP);

    stream_set_blocking($server, false);
    Kernel::spawn(function() use ($server) {
        while (true) {
            $conn = @stream_socket_accept($server, 0);
            if ($conn) {
                fwrite($conn, "AsyncTCP\n");
                fclose($conn);
                break;
            }
            Time::sleep(10);
        }
    });
    
    echo "[Test] Connecting via fopen('tcp://...')...\n";
    $fp = fopen("tcp://127.0.0.1:8083", "r");
    if ($fp) {
        echo "[Test] Connected. Reading...\n";
        $data = fread($fp, 1024);
        echo "[Test] Received: $data";
        if (trim($data) === "AsyncTCP") {
            echo "[Test] TCP Hook PASSED.\n";
        } else {
            echo "[Test] TCP Hook FAILED (Wrong data).\n";
        }
        fclose($fp);
    } else {
        echo "[Test] TCP Hook FAILED (Connect failed).\n";
    }
    
    // Clean up wrapper? No explicit unregister needed for exit.
});
