<?php

namespace Async\Network\Udp;

use Async\Kernel\Network\UdpSocket as KernelUdpSocket;
use Fiber;

class Socket
{
    private KernelUdpSocket $inner;

    public function __construct(KernelUdpSocket $inner)
    {
        $this->inner = $inner;
    }

    public static function bind(string $addr): self
    {
        $future = KernelUdpSocket::bind($addr);
        $kernelSocket = Fiber::suspend($future);
        if (!$kernelSocket) {
            throw new \RuntimeException("Failed to bind to $addr");
        }
        return new self($kernelSocket);
    }

    public function recvFrom(int $length = 65535): array|false
    {
        // Rust returns [data, addr]
        $future = $this->inner->recv_from($length);
        return Fiber::suspend($future);
    }

    public function sendTo(string $data, string $addr): int|false
    {
        $future = $this->inner->send_to($data, $addr);
        return Fiber::suspend($future);
    }
}