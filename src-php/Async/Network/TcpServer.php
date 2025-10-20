<?php

namespace Async\Network;

use AsyncTcpListener;
use RustFuture;
use Fiber;

class TcpServer
{
    private AsyncTcpListener $listener;

    public function __construct(string $host, int $port)
    {
        $addr = "$host:$port";
        $future = AsyncTcpListener::bind($addr);
        $this->listener = Fiber::suspend($future);
        
        if (!$this->listener) {
            throw new \RuntimeException("Failed to bind to $addr");
        }
    }

    public function accept(): TcpSocket
    {
        $future = $this->listener->accept();
        $resource = Fiber::suspend($future);
        
        if (!$resource) {
            throw new \RuntimeException("Accept failed");
        }
        
        return new TcpSocket($resource);
    }
}
