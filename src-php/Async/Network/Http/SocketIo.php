<?php

namespace Async\Network\Http;

use Async\Kernel\Network\Http\SocketIo as KernelSocketIo;
use Async\Kernel\Network\Http\SocketIo\Socket as KernelSocket;

class SocketIo
{
    private KernelSocketIo $kernel;

    public function __construct()
    {
        $this->kernel = new KernelSocketIo();
    }

    public function onConnection(string $namespace, callable $callback): void
    {
        $this->kernel->onConnection($namespace, function(KernelSocket $kernelSocket) use ($callback) {
            $socket = new \Async\Network\Http\SocketIo\Socket($kernelSocket);
            $callback($socket);
        });
    }
    
    /**
     * @internal
     */
    public function getKernel(): KernelSocketIo
    {
        return $this->kernel;
    }
}
