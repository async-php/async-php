<?php

require_once __DIR__ . '/../vendor/autoload.php';

use Async\Kernel;
use Async\Time;
use Async\Network\TcpServer;
use Async\Network\TcpSocket;

Kernel::run(function () {
    echo "Starting TCP Echo Server on 127.0.0.1:8080...\n";
    
    try {
        $server = new TcpServer('127.0.0.1', 8080);
    } catch (\Exception $e) {
        echo "Failed to bind: " . $e->getMessage() . "\n";
        return;
    }

    // Accept loop
    while (true) {
        try {
            $socket = $server->accept();
            echo "New client connected!\n";

            // Spawn a new fiber to handle this client concurrently
            Kernel::spawn(function () use ($socket) {
                handle_client($socket);
            });

        } catch (\Exception $e) {
            echo "Accept error: " . $e->getMessage() . "\n";
            Time::sleep(100); // Prevent tight loop on error
        }
    }
});

function handle_client(TcpSocket $socket) {
    try {
        $socket->write("Welcome to Async PHP Echo Server!\n");
        
        while (true) {
            $data = $socket->read(1024);
            
            if ($data === '') {
                echo "Client disconnected.\n";
                break;
            }
            
            echo "Received: " . trim($data) . "\n";
            
            // Simulate some "heavy" async processing (DB, API, etc.)
            // Time::sleep(10);
            
            if (trim($data) === 'bye') {
                $socket->write("Goodbye!\n");
                break;
            }
            
            $socket->write("Echo: " . $data);
        }
    } catch (\Exception $e) {
        echo "Client error: " . $e->getMessage() . "\n";
    } finally {
        $socket->close();
    }
}
