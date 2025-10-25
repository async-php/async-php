<?php

namespace Async\Network;

use Async\Kernel\Network\UnixListener;
use Fiber;

class UnixServer
{
    private UnixListener $listener;

    public function __construct(string $path)
    {
        $future = UnixListener::bind($path);
        $this->listener = Fiber::suspend($future);
        
        if (!$this->listener) {
            throw new \RuntimeException("Failed to bind to $path");
        }
    }

    public function accept(): UnixSocket
    {
        $future = $this->listener->accept();
        $resource = Fiber::suspend($future);
        
        if (!$resource) {
            throw new \RuntimeException("Accept failed");
        }
        
        return new UnixSocket($resource);
    }
}
