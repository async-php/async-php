<?php

namespace Async\Http;

use Async\Kernel\Network\Http\Server as HttpServer;
use Async\Kernel\Network\Http\Request as HttpRequest;
use Async\Kernel\Network\Http\Response as HttpResponse;
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
        // Handler: function(HttpRequest $req): HttpResponse|string
        $future = HttpServer::listen($this->addr, $handler);
        Fiber::suspend($future);
    }
}
