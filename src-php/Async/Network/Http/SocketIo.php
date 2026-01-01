<?php

namespace Async\Network\Http;

use Async\Kernel\Network\Http\SocketIo as KernelSocketIo;
use Async\Kernel\Network\Http\SocketIo\Socket as KernelSocket;
use Async\Network\Http\SocketIo\AdapterInterface;
use Async\Network\Http\SocketIo\Socket;

class SocketIo
{
    private KernelSocketIo $kernel;

    public function __construct(?AdapterInterface $adapter = null)
    {
        $this->kernel = new KernelSocketIo($adapter);
        if ($adapter) {
            $adapter->setKernel($this->kernel);
        }
    }

    public function onConnection(string $namespace, callable $callback): void
    {
        $this->kernel->onConnection($namespace, function(KernelSocket $kernelSocket) use ($callback) {
            $callback(new Socket($kernelSocket));
        });
    }

    public function getKernel(): KernelSocketIo
    {
        return $this->kernel;
    }
}