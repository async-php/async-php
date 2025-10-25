<?php

namespace Async\Network;

use Async\Kernel\Network\UdpSocket as KernelUdpSocket;
use Fiber;

class UdpSocket
{
    private KernelUdpSocket $socket;

    private function __construct(KernelUdpSocket $socket)
    {
        $this->socket = $socket;
    }

    public static function bind(string $host, int $port): ?self
    {
        $addr = "$host:$port";
        $socket = Fiber::suspend(KernelUdpSocket::bind($addr));
        
        if (!$socket) return null;
        return new self($socket);
    }

    public function sendTo(string $data, string $host, int $port): int|false
    {
        $addr = "$host:$port";
        return Fiber::suspend($this->socket->sendTo($data, $addr));
    }

    public function recvFrom(int $length = 65535, &$peer = null): string|false
    {
        // Rust returns [data, address]
        $result = Fiber::suspend($this->socket->recvFrom($length));
        
        if (is_array($result)) {
            $peer = $result[1];
            return $result[0];
        }
        
        return false;
    }
}
