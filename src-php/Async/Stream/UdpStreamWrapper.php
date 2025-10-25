<?php

namespace Async\Stream;

use Async\Kernel\Network\UdpSocket;
use Fiber;

class UdpStreamWrapper
{
    public $context;
    private ?UdpSocket $socket = null;
    private ?string $remoteAddr = null;

    public function stream_open(string $path, string $mode, int $options, ?string &$opened_path): bool
    {
        $parts = parse_url($path);
        if (!$parts || !isset($parts['host'], $parts['port'])) {
            return false;
        }
        
        $this->remoteAddr = $parts['host'] . ':' . $parts['port'];
        
        // Bind to a random local port to allow sending
        $future = UdpSocket::bind("0.0.0.0:0");
        $this->socket = Fiber::suspend($future);
        
        return $this->socket !== null;
    }

    public function stream_read(int $count): string|false
    {
        if ($this->socket) {
            // recv_from returns [data, addr]
            // We discard addr in stream_read since streams are byte streams usually
            // Ideally stream_socket_recvfrom handles this but via wrapper stream_read is just data.
            $res = Fiber::suspend($this->socket->recvFrom($count));
            if (is_array($res)) {
                return $res[0];
            }
        }
        return false;
    }

    public function stream_write(string $data): int|false
    {
        if ($this->socket && $this->remoteAddr) {
            return Fiber::suspend($this->socket->sendTo($data, $this->remoteAddr));
        }
        return false;
    }

    public function stream_eof(): bool
    {
        return false;
    }

    public function stream_close(): void
    {
        $this->socket = null;
    }
}
