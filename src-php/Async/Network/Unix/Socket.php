<?php

namespace Async\Network\Unix;

use Async\Kernel\Network\UnixStream as KernelUnixStream;
use Fiber;

class Socket
{
    private KernelUnixStream $inner;

    public function __construct(KernelUnixStream $inner)
    {
        $this->inner = $inner;
    }

    public static function connect(string $path): self
    {
        $future = KernelUnixStream::connect($path);
        $kernelStream = Fiber::suspend($future);
        if (!$kernelStream) {
            throw new \RuntimeException("Failed to connect to $path");
        }
        return new self($kernelStream);
    }

    public function read(int $length = 1024): string|false
    {
        $future = $this->inner->read($length);
        return Fiber::suspend($future);
    }

    public function write(string $data): int|false
    {
        $future = $this->inner->write($data);
        return Fiber::suspend($future);
    }

    public function close(): bool
    {
        $future = $this->inner->close();
        return (bool)Fiber::suspend($future);
    }

    /**
     * Get the local socket address
     */
    public function localAddr(): string
    {
        return $this->inner->local_addr();
    }

    /**
     * Get the remote peer socket address
     */
    public function peerAddr(): string
    {
        return $this->inner->peer_addr();
    }

    /**
     * Get peer credentials (process ID, user ID, group ID)
     * Only supported on Linux, macOS, iOS, FreeBSD, NetBSD, OpenBSD
     * @return array|null Array with keys: pid, uid, gid, or null if not supported
     */
    public function peerCred(): ?array
    {
        return $this->inner->peer_cred() ?: null;
    }
}