<?php
use Async\Kernel;
use Async\Network\Http\Server;
use Async\Network\Http\SocketIo;
use Async\Network\Http\SocketIo\Socket;
use Async\Time;
use Async\Network\Http\Request;
use Async\Network\Http\Response;
use Async\Kernel\Logger;

require_once __DIR__ . '/../../../vendor/autoload.php';

// Store sockets that want lag simulation
// Using a simple array with socket ID as key
$laggySockets = [];

Kernel::run(function () use (&$laggySockets) {
    Logger::init(['level' => 'debug']);
    $io = new SocketIo();

    $io->onConnection('/', function (Socket $socket) use (&$laggySockets) {
        echo "New client connected: " . $socket->id() . "\n";
        
        // Broadcast that a new user joined
        $socket->broadcast('system_message', [
            'type' => 'info',
            'text' => 'A new user joined the chat'
        ]);

        $socket->on('toggle_lag', function ($enabled, Socket $s) use (&$laggySockets) {
            if ($enabled) {
                $laggySockets[$s->id()] = true;
                echo "Client " . $s->id() . " enabled lag simulation.\n";
            } else {
                unset($laggySockets[$s->id()]);
                echo "Client " . $s->id() . " disabled lag simulation.\n";
            }
        });

        $socket->on('chat_message', function ($msg, Socket $s) use (&$laggySockets) {
            $isLaggy = isset($laggySockets[$s->id()]);
            
            if ($isLaggy) {
                echo "Simulating lag for " . $s->id() . " (2s)...\n";
                Time::sleep(2.0);
            }

            echo "Broadcasting message from " . $s->id() . ": " . $msg . "\n";
            
            // Broadcast to everyone else
            $s->broadcast('chat_message', [
                'user' => substr($s->id(), 0, 5), // Simple username
                'text' => $msg,
                'time' => date('H:i:s')
            ]);
            
            // Send back to sender (ack)
            $s->emit('chat_message', [
                'user' => 'Me',
                'text' => $msg,
                'time' => date('H:i:s')
            ]);
        });
        
        $socket->on('typing', function ($data, Socket $s) {
            $s->broadcast('typing', [
                'user' => substr($s->id(), 0, 5)
            ]);
        });
    });

    $server = new Server();
    $server->withSocketIo($io);
    
    echo "Chat server listening on http://0.0.0.0:8080\n";
    echo "Open http://localhost:8080 in your browser\n";

    // Serve static files for the chat client
    $server->listenAndServe('0.0.0.0:8080', function (Request $req): Response {
        $path = $req->path();
        $res = new Response();

        if ($path === '/' || $path === '/index.html') {
            $content = file_get_contents(__DIR__ . '/index.html');
            $res->withHeader('Content-Type', 'text/html');
            $res->withBody($content);
        } else {
            $res->withStatus(404);
            $res->withBody('Not Found');
        }
        return $res;
    });
});
