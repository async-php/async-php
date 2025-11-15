<?php

namespace Async\Network\Http;

use Async\Kernel\Network\Http\HttpResponse as KernelResponse;
use Psr\Http\Message\StreamInterface;
use Fiber;

/**
 * PSR-7 Stream implementation for HTTP response bodies
 *
 * This wraps the kernel HttpResponse and provides lazy body reading.
 * The body is read from the kernel response on first access.
 */
class Stream implements StreamInterface
{
    private KernelResponse $kernelResponse;
    private ?string $contents = null;
    private int $position = 0;
    private bool $seekable = true;
    private bool $readable = true;
    private bool $writable = false;

    public function __construct(KernelResponse $kernelResponse)
    {
        $this->kernelResponse = $kernelResponse;
    }

    /**
     * Reads body from kernel response if not already cached
     */
    private function ensureContents(): void
    {
        if ($this->contents !== null) {
            return;
        }

        // Read body as text from kernel response
        $future = $this->kernelResponse->text();
        $this->contents = Fiber::suspend($future);
    }

    public function __toString(): string
    {
        try {
            $this->ensureContents();
            return $this->contents ?? '';
        } catch (\Throwable $e) {
            return '';
        }
    }

    public function close(): void
    {
        // No-op: kernel manages the underlying connection
    }

    public function detach()
    {
        $this->contents = null;
        $this->readable = false;
        $this->seekable = false;
        return null;
    }

    public function getSize(): ?int
    {
        $this->ensureContents();
        return $this->contents !== null ? strlen($this->contents) : null;
    }

    public function tell(): int
    {
        return $this->position;
    }

    public function eof(): bool
    {
        $this->ensureContents();
        return $this->position >= strlen($this->contents ?? '');
    }

    public function isSeekable(): bool
    {
        return $this->seekable;
    }

    public function seek($offset, $whence = SEEK_SET): void
    {
        if (!$this->seekable) {
            throw new \RuntimeException('Stream is not seekable');
        }

        $this->ensureContents();
        $length = strlen($this->contents ?? '');

        switch ($whence) {
            case SEEK_SET:
                $newPosition = $offset;
                break;
            case SEEK_CUR:
                $newPosition = $this->position + $offset;
                break;
            case SEEK_END:
                $newPosition = $length + $offset;
                break;
            default:
                throw new \InvalidArgumentException('Invalid whence value');
        }

        if ($newPosition < 0 || $newPosition > $length) {
            throw new \RuntimeException('Invalid seek position');
        }

        $this->position = $newPosition;
    }

    public function rewind(): void
    {
        $this->seek(0);
    }

    public function isWritable(): bool
    {
        return $this->writable;
    }

    public function write($string): int
    {
        throw new \RuntimeException('Stream is not writable');
    }

    public function isReadable(): bool
    {
        return $this->readable;
    }

    public function read($length): string
    {
        if (!$this->readable) {
            throw new \RuntimeException('Stream is not readable');
        }

        $this->ensureContents();
        $data = substr($this->contents ?? '', $this->position, $length);
        $this->position += strlen($data);
        return $data;
    }

    public function getContents(): string
    {
        if (!$this->readable) {
            throw new \RuntimeException('Stream is not readable');
        }

        $this->ensureContents();
        $data = substr($this->contents ?? '', $this->position);
        $this->position = strlen($this->contents ?? '');
        return $data;
    }

    public function getMetadata($key = null)
    {
        $metadata = [
            'seekable' => $this->seekable,
            'readable' => $this->readable,
            'writable' => $this->writable,
        ];

        if ($key === null) {
            return $metadata;
        }

        return $metadata[$key] ?? null;
    }
}
