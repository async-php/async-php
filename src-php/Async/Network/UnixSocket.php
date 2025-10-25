<?php

namespace Async\Network;

use Async\Kernel\Network\UnixStream;
use Fiber;

class UnixSocket
{
    private UnixStream $stream;

    public function __construct(UnixStream $stream)
    {
        $this->stream = $stream;
    }

    public static function connect(string $path): ?self
    {
        $stream = Fiber::suspend(UnixStream::connect($path));
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
