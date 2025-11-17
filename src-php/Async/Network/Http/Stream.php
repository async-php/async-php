<?php

namespace Async\Network\Http;

use Async\Kernel\Network\Http\HttpResponse as KernelResponse;
use Async\Kernel\Network\Http\HttpResponseBody;
use Psr\Http\Message\StreamInterface;
use Fiber;

/**
 * PSR-7 Stream implementation for HTTP response bodies
 *
 * This implementation supports two modes:
 * 1. Streaming mode (default): Reads data on-demand without buffering entire response
 * 2. Seekable mode: Buffers entire response to support seek operations
 *
 * For large responses, streaming mode is much more memory-efficient.
 * Seekable mode is activated automatically when seek() is called.
 */
class Stream implements StreamInterface
{
    private KernelResponse $kernelResponse;
    private ?HttpResponseBody $streamBody = null;
    private ?string $bufferedContents = null;
    private int $position = 0;
    private bool $readable = true;
    private bool $writable = false;
    private bool $eof = false;

    public function __construct(KernelResponse $kernelResponse)
    {
        $this->kernelResponse = $kernelResponse;
    }

    /**
     * Get the streaming body (lazy initialization)
     */
    private function getStreamBody(): ?HttpResponseBody
    {
        if ($this->streamBody === null && $this->bufferedContents === null) {
            try {
                $this->streamBody = $this->kernelResponse->stream();
            } catch (\Throwable $e) {
                // Body already consumed or error
                $this->eof = true;
                return null;
            }
        }
        return $this->streamBody;
    }

    /**
     * Reads and buffers entire body (required for seek operations)
     */
    private function ensureBuffered(): void
    {
        if ($this->bufferedContents !== null) {
            return;
        }

        // If we have a stream body, read all from it
        if ($this->streamBody !== null) {
            $future = $this->streamBody->readAll();
            $this->bufferedContents = Fiber::suspend($future) ?? '';
            $this->streamBody = null; // Release stream
            return;
        }

        // Otherwise, read from kernel response
        try {
            $future = $this->kernelResponse->text();
            $this->bufferedContents = Fiber::suspend($future) ?? '';
        } catch (\Throwable $e) {
            $this->bufferedContents = '';
            $this->eof = true;
        }
    }

    public function __toString(): string
    {
        try {
            // For __toString we need the full contents
            $this->ensureBuffered();
            return $this->bufferedContents ?? '';
        } catch (\Throwable $e) {
            return '';
        }
    }

    public function close(): void
    {
        $this->streamBody = null;
        $this->bufferedContents = null;
        $this->readable = false;
    }

    public function detach()
    {
        $this->streamBody = null;
        $this->bufferedContents = null;
        $this->readable = false;
        return null;
    }

    public function getSize(): ?int
    {
        // Try to get Content-Length from response headers without reading body
        $contentLength = $this->kernelResponse->contentLength();
        if ($contentLength !== null) {
            return (int)$contentLength;
        }

        // If buffered, return buffered size
        if ($this->bufferedContents !== null) {
            return strlen($this->bufferedContents);
        }

        // Unknown size for streaming responses
        return null;
    }

    public function tell(): int
    {
        return $this->position;
    }

    public function eof(): bool
    {
        // If buffered, check against buffered content
        if ($this->bufferedContents !== null) {
            return $this->position >= strlen($this->bufferedContents);
        }

        // If streaming, EOF is tracked by read operations
        return $this->eof;
    }

    public function isSeekable(): bool
    {
        // Seekable only if we've buffered the content
        return $this->bufferedContents !== null;
    }

    public function seek($offset, $whence = SEEK_SET): void
    {
        // Seeking requires buffering the entire response
        $this->ensureBuffered();

        $length = strlen($this->bufferedContents ?? '');

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

        if ($length <= 0) {
            return '';
        }

        // If buffered, read from buffer
        if ($this->bufferedContents !== null) {
            $data = substr($this->bufferedContents, $this->position, $length);
            $this->position += strlen($data);
            return $data;
        }

        // Streaming mode: read from stream body
        $streamBody = $this->getStreamBody();
        if ($streamBody === null) {
            $this->eof = true;
            return '';
        }

        try {
            $future = $streamBody->read($length);
            $data = Fiber::suspend($future);

            if ($data === null || $data === '') {
                $this->eof = true;
                return '';
            }

            $this->position += strlen($data);
            return $data;
        } catch (\Throwable $e) {
            $this->eof = true;
            return '';
        }
    }

    public function getContents(): string
    {
        if (!$this->readable) {
            throw new \RuntimeException('Stream is not readable');
        }

        // If buffered, return remaining buffered content
        if ($this->bufferedContents !== null) {
            $data = substr($this->bufferedContents, $this->position);
            $this->position = strlen($this->bufferedContents);
            return $data;
        }

        // Streaming mode: read all remaining from stream
        $streamBody = $this->getStreamBody();
        if ($streamBody === null) {
            $this->eof = true;
            return '';
        }

        try {
            $future = $streamBody->readAll();
            $data = Fiber::suspend($future) ?? '';
            $this->position += strlen($data);
            $this->eof = true;
            return $data;
        } catch (\Throwable $e) {
            $this->eof = true;
            return '';
        }
    }

    public function getMetadata($key = null)
    {
        $metadata = [
            'seekable' => $this->isSeekable(),
            'readable' => $this->readable,
            'writable' => $this->writable,
            'mode' => $this->bufferedContents !== null ? 'buffered' : 'streaming',
        ];

        if ($key === null) {
            return $metadata;
        }

        return $metadata[$key] ?? null;
    }
}
