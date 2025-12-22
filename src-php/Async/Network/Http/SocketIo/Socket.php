<?php

namespace Async\Network\Http\SocketIo;

use Async\Kernel\Network\Http\SocketIo\Socket as KernelSocket;

class Socket
{
    private KernelSocket $kernel;

    public function __construct(KernelSocket $kernel)
    {
        $this->kernel = $kernel;
    }

    public function id(): string
    {
        return $this->kernel->id();
    }

    public function emit(string $event, mixed $data = null): void
    {
        $this->kernel->emit($event, $data);
    }

    public function join(string $room): void
    {
        $this->kernel->join($room);
    }

    public function leave(string $room): void
    {
        $this->kernel->leave($room);
    }

    public function broadcast(string $event, mixed $data = null): void
    {
        $this->kernel->broadcast($event, $data);
    }

    public function to(string $room, string $event, mixed $data = null): void
    {
        $this->kernel->to($room, $event, $data);
    }

    public function on(string $event, callable $callback): void
    {
        // Wrap the callback to convert arguments if necessary
        // The data is JSON/array, so it's fine.
        // The second argument is the Socket itself (in Rust: emit to same socket).
        // Wait, current Rust impl: `handler.0.try_call(vec![&data_zval, &socket_zval])`
        // So the callback receives ($data, $kernelSocket).
        
        $this->kernel->on($event, function($data, $kernelSocket) use ($callback) {
            // We should provide the wrapped socket to the callback as well
            // But we can just reuse $this if it's the same socket? 
            // Actually, the socket passed in the event callback IS the same socket ref.
            // But creating a new wrapper is safer to avoid state issues if any.
            // Or we can just pass $this?
            // The Rust code creates a NEW `AsyncSocket` struct for the callback.
            // So we should wrap it.
            
            $socket = new Socket($kernelSocket);
            $callback($data, $socket);
        });
    }
}
