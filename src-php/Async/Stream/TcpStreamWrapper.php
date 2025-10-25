<?php

namespace Async\Stream;

use Async\TcpStream;
use Async\TcpListener;
use Fiber;

class TcpStreamWrapper
{
    /** @var resource */
    public $context;
    
    private ?TcpStream $socket = null;
    private ?TcpListener $listener = null;
    private bool $eof = false;

    public function stream_open(string $path, string $mode, int $options, ?string &$opened_path): bool
    {
        $parts = parse_url($path);
        if (!$parts || !isset($parts['host'], $parts['port'])) {
            return false;
        }
        
        $addr = $parts['host'] . ':' . $parts['port'];
        
        // Connect (Client Mode)
        // STREAM_CLIENT_CONNECT is set by stream_socket_client.
        // fopen passes 0 options by default.
        
        if (($options & STREAM_CLIENT_CONNECT) || $options === 0) {
             $future = TcpStream::connect($addr);
             $this->socket = Fiber::suspend($future);
             return $this->socket !== null;
        }
        
        return false;
    }

    public function stream_read(int $count): string|false
    {
        if ($this->socket) {
            $data = Fiber::suspend($this->socket->read($count));
            if ($data === "" || $data === false) {
                $this->eof = true;
            }
            return $data;
        }
        return false;
    }

    public function stream_write(string $data): int|false
    {
        if ($this->socket) {
            return Fiber::suspend($this->socket->write($data));
        }
        return false;
    }

    public function stream_eof(): bool
    {
        return $this->eof;
    }

    public function stream_close(): void
    {
        if ($this->socket) {
            Fiber::suspend($this->socket->close());
            $this->socket = null;
        }
        // Listener close?
    }
    
    // For stream_socket_server? 
    // Documentation says generic wrappers don't easily support xport functions like stream_socket_server 
    // unless they implement specific cast or internal structures.
    // So stream_socket_server might NOT be hooked by just registering 'tcp'.
    // It usually requires registering a "transport" (stream_xport_register), not just a wrapper.
    // PHP userland cannot register transports easily.
    // So we might only be able to hook stream_socket_client via this method if it falls back to wrapper.
    // But let's stick to Client for now.
}
