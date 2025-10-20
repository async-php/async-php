<?php

namespace Async\Network;

use AsyncTcpStream;
use RustFuture;
use Fiber;

class TcpSocket
{
    private AsyncTcpStream $stream;

    public function __construct(AsyncTcpStream $stream)
    {
        $this->stream = $stream;
    }

    public function read(int $length = 1024): string
    {
        $future = $this->stream->read($length);
        $result = Fiber::suspend($future);
        return $result === false ? '' : $result;
    }

    public function write(string $data): int
    {
        $future = $this->stream->write($data);
        $result = Fiber::suspend($future);
        return $result === false ? 0 : $result;
    }

    public function close(): void
    {
        $future = $this->stream->close();
        Fiber::suspend($future);
    }
}
