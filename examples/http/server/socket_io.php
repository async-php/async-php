<?php

use Async\Network\Http\Server;
use Async\Network\Http\Request;
use Async\Network\Http\Response;
use Async\Network\Http\SocketIo;
use Async\Network\Http\SocketIo\Socket;
use Async\Kernel;

require_once __DIR__ . '/../../../vendor/autoload.php';

Kernel::run(function () {
    // Create Socket.IO instance
    $io = new SocketIo();

    // Handle connections on "/" namespace
    $io->onConnection('/', function (Socket $socket) {
        echo "New connection: " . $socket->id() . "\n";

        $socket->emit('message', 'Welcome to Async PHP Socket.IO!');

        $socket->on('chat', function ($data, Socket $s) {
            echo "Received chat: " . json_encode($data) . "\n";
            // Echo back
            $s->emit('chat', $data);
        });

        $socket->on('join', function ($room, Socket $s) {
            echo "Joining room: $room\n";
            $s->join($room);
            $s->emit('joined', "You joined $room");
        });
    });

    // Create HTTP Server
    $server = new Server();
    $server->withSocketIo($io);

    echo "Listening on http://127.0.0.1:8080\n";
    echo "Try connecting with a Socket.IO v4 client.\n";

    $server->listenAndServe('127.0.0.1:8080', function (Request $req): Response {
        $res = new Response();
        $res->withBody('Hello World (HTTP)');
        return $res;
    });
});

