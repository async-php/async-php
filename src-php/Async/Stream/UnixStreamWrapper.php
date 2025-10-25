<?php

namespace Async\Stream;

use Async\Kernel\Network\UnixStream;
use Fiber;

class UnixStreamWrapper
{
    public $context;
    private ?UnixStream $socket = null;
    private bool $eof = false;

    public function stream_open(string $path, string $mode, int $options, ?string &$opened_path): bool
    {
        if (!str_starts_with($path, 'unix://')) {
            return false;
        }
        $socketPath = substr($path, 7);
        
        $future = UnixStream::connect($socketPath);
        $this->socket = Fiber::suspend($future);
        
        if (!$this->socket) {
            // Debug
            // echo "UnixStreamWrapper: Connect failed to $socketPath\n";
            return false;
        }
        
        return true;
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
    }
}
