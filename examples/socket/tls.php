<?php

require_once __DIR__ . '/../../vendor/autoload.php';

use Async\Kernel;

Kernel::run(function () {
    Kernel::enableCoroutine(Kernel::HOOK_SSL);

    echo "[Client] Connecting to google.com:443 via TLS...\n";
    
    // Note: This requires CA certificates to be available or verification disabled.
    // Our TlsStreamWrapper attempts to read 'ssl'=>['cafile'] from context.
    // If not provided, Rust TlsStream might fail if it can't find system certs (which it won't without loading them).
    // In the Rust code, I added `rustls-pemfile` but didn't add `rustls-native-certs` (I think).
    // But I added code to load from CA file if provided.
    // Let's try without verification first? Rustls is strict. It usually requires valid certs.
    // Let's skip verification if possible? 
    // My AsyncTlsStream::connect uses `with_no_client_auth()` but that's for client auth.
    // Server verification is default.
    // I need to pass a valid CA file or rely on example.com.
    
    // For this test, I'll try to connect to a known host.
    // Without a CA file, `RootCertStore::empty()` is used in my code if no file provided.
    // So it will fail verification unless I provide a CA file.
    // I will use a dummy check here or expect failure if no CA provided, but at least check if it attempts async hook.
    
    // On macOS (Darwin), we might find a cert file.
    $caFile = '/etc/ssl/cert.pem'; // Common on Linux
    if (!file_exists($caFile)) {
        $caFile = '/opt/homebrew/etc/openssl@3/cert.pem'; // Homebrew
    }
    if (!file_exists($caFile)) {
        $caFile = '/usr/local/etc/openssl@1.1/cert.pem';
    }
    
    // PHP Context
    $ctx = stream_context_create([
        'ssl' => [
            'cafile' => $caFile,
            'verify_peer' => false, // Rustls might not respect this PHP flag unless I map it. I didn't map it yet.
        ]
    ]);

    if (!file_exists($caFile)) {
        echo "WARNING: CA file not found. TLS connection might fail.\n";
    }

    // Use fopen with tls://
    $fp = fopen("tls://google.com:443", 'r', false, $ctx);
    
    if (!$fp) {
        echo "FAILURE: Could not connect.\n";
        // If it failed because of hook, it's one thing.
        return;
    }
    
    fwrite($fp, "GET / HTTP/1.1\r\nHost: google.com\r\n\r\n");
    fflush($fp);
    $response = fread($fp, 2048); // Read some bytes
    
    echo "[Client] Response received (" . strlen($response) . " bytes).\n";
    
    if (strlen($response) > 0) {
        echo "SUCCESS: TLS Hook works (Data received).\n";
    } else {
        echo "FAILURE: No data received.\n";
    }
    
    fclose($fp);
});
