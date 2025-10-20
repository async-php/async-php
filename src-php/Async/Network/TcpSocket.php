<?php

namespace Async\Network;

use AsyncTcpStream;
use Fiber;

class TcpSocket
{
    private AsyncTcpStream $stream;

    public function __construct(AsyncTcpStream $stream)
    {
        $this->stream = $stream;
    }

    public static function connect(string $host, int $port): ?self
    {
        $addr = "$host:$port";
        $stream = Fiber::suspend(AsyncTcpStream::connect($addr));
        
        if (!$stream) return null;
        return new self($stream);
    }

    public function read(int $length = 1024): string|false
    {
        return Fiber::suspend($this->stream->read($length));
    }

    public function write(string $data): int|false
    {
        return Fiber::suspend($this->stream->write($data));
    }

    public function close(): bool
    {
        return Fiber::suspend($this->stream->close());
    }

    public function getPeerName(): string
    {
        return $this->stream->peerAddr();
    }

    public function setNoDelay(bool $enable): bool
    {
        return $this->stream->setNodelay($enable);
    }
}