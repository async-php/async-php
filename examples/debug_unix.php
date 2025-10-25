<?php

require_once __DIR__ . '/../vendor/autoload.php';

use Async\Kernel;
use Async\Network\UnixSocket; // Use explicit class if wrapper fails

Kernel::run(function () {
    // We will try to connect using the class directly to debug if it's wrapper issue or socket issue.
    $sockPath = '/tmp/async_test.sock';
    
    // Ensure server is running (we assume the previous test left it running or we start one?)
    // The previous test started a server in a spawn, but then exited.
    // Wait, `examples/unix_test.php` calls `exit(0)` at the end.
    // So the server is killed when the process exits.
    // But `examples/unix_test.php` IS the process.
    // The server is spawned.
    
    // Let's debug by using explicit class in the client part of `unix_test.php` instead of fopen for a moment?
    // No, user wants stream tests.
    
    // Let's look at `src/net.rs`:
    // `AsyncUnixListener::bind` -> `tokio::net::UnixListener::bind(path)`.
    // `AsyncUnixStream::connect` -> `tokio::net::UnixStream::connect(path)`.
    
    // Maybe the socket file is not ready? I increased sleep to 500ms.
    
    // Is it possible that `parse_url` on `unix:///tmp...` behaves differently?
    // `parse_url('unix:///tmp/sock')` -> scheme='unix', path='/tmp/sock'.
    // `parse_url('unix://tmp/sock')` -> scheme='unix', host='tmp', path='/sock'.
    // My test uses `unix:///tmp/async_test.sock`. So path should be `/tmp/async_test.sock`.
    
    // I suspect `tokio::net::UnixStream::connect` is failing.
});
