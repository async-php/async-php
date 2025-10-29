<?php

namespace Async\Network\Http;

use Async\Kernel\Network\Http\HttpServer as KernelServer;

class Server
{
    public static function listen(string $addr, callable $handler, array $config = []): void
    {
        $server = new KernelServer();
        $server->bind($addr);

        $wrappedHandler = function ($kernelRequest) use ($handler) {
            $request = new Request($kernelRequest);
            $response = $handler($request);

            if ($response instanceof Response) {
                return $response->getKernel();
            }

            return $response;
        };

        // Register handler for root path by default
        $server->handle('/', $wrappedHandler);

        $future = $server->start();
        \Fiber::suspend($future);
    }
}
