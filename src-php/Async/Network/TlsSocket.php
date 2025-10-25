<?php

namespace Async\Network;

use Async\Kernel\Network\TlsStream;
use Fiber;

class TlsSocket
{
    private TlsStream $stream;

    public function __construct(TlsStream $stream)
    {
        $this->stream = $stream;
    }

    public static function connect(string $host, int $port, ?string $caFile = null): ?self
    {
        $stream = Fiber::suspend(TlsStream::connect($host, $port, $caFile));
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
}
