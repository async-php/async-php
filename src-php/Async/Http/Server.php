<?php

namespace Async\Http;

use AsyncHttpServer;
use AsyncHttpRequest;
use AsyncHttpResponse;
use Fiber;

class Server
{
    private string $addr;

    public function __construct(string $host, int $port)
    {
        $this->addr = "$host:$port";
    }

    public function handle(callable $handler): void
    {
        // delegating to Rust Hyper server
        // Handler: function(AsyncHttpRequest $req): AsyncHttpResponse|string
        $future = AsyncHttpServer::listen($this->addr, $handler);
        Fiber::suspend($future);
    }
}
