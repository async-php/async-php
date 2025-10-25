<?php

namespace Async\Stream;

use Async\Kernel\Network\TlsStream;
use Fiber;

class TlsStreamWrapper
{
    public $context;
    private ?TlsStream $socket = null;
    private bool $eof = false;

    public function stream_open(string $path, string $mode, int $options, ?string &$opened_path): bool
    {
        // tls://host:port or ssl://host:port
        $parts = parse_url($path);
        if (!$parts || !isset($parts['host'], $parts['port'])) {
            return false;
        }
        
        $host = $parts['host'];
        $port = $parts['port'];
        
        // TODO: Extract CA file from context if provided
        $caFile = null;
        if ($this->context) {
            $opts = stream_context_get_options($this->context);
            if (isset($opts['ssl']['cafile'])) {
                $caFile = $opts['ssl']['cafile'];
            }
        }
        
        $future = TlsStream::connect($host, $port, $caFile);
        $this->socket = Fiber::suspend($future);
        
        return $this->socket !== null;
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
