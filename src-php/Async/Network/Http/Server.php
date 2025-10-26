<?php

namespace Async\Network\Http;

use Async\Kernel\Network\Http\Server as KernelServer;

class Server
{
    public static function listen(string $addr, callable $handler, array $config = []): void
    {
        $wrappedHandler = function ($kernelRequest) use ($handler) {
            $request = new Request($kernelRequest);
            $response = $handler($request);
            
            if ($response instanceof Response) {
                return $response->getKernel();
            }
            
            return $response; // Fallback for simple strings or KernelResponse
        };

        $future = KernelServer::listen($addr, $wrappedHandler, $config);
        \Fiber::suspend($future);
    }
}