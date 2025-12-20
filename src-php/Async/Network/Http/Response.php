<?php

namespace Async\Network\Http;

use Async\IO;
use Async\IO\Reader;
use Async\IO\TokioIO;
use Async\Kernel\IO\BytesReader;
use Async\Kernel\Network\Http\HttpResponse as KernelResponse;
use Async\IO\Wrapper\ReaderWrapper;

/**
 * HTTP Response (userland) - kernel-agnostic.
 *
 * - Does not hold Async\Kernel\Network\Http\HttpResponse directly.
 * - Convert via Response::fromKernelResponse() and Response->toKernelResponse().
 * - Body is represented as Async\IO\Reader for streaming.
 *
 * Usage (Server):
 * ```php
 * $response = new Response();
 * $response->setStatus(200);
 * $response->setHeader('Content-Type', 'application/json');
 * $response->setBody(json_encode(['message' => 'Hello']));
 * return $response;
 * ```
 *
 * Usage (Client):
 * ```php
 * $response = $client->get('https://example.com');
 * echo $response->status();
 * echo $response->text();
 * $data = $response->json();
 * ```
 */
class Response
{
    private int $status = 200;
    private ?string $version = null;

    /** @var array<string, list<string>> */
    private array $headers = [];

    private ?Reader $body = null;
    private ?string $cachedText = null;

    /**
     * Create a Response
     */
    public function __construct(int $status = 200, mixed $headersOrBody = [], mixed $body = null)
    {
        $this->status = $status;

        if (is_array($headersOrBody)) {
            foreach ($headersOrBody as $name => $value) {
                if (is_array($value)) {
                    foreach ($value as $v) {
                        $this->appendHeader((string)$name, (string)$v);
                    }
                } else {
                    $this->appendHeader((string)$name, (string)$value);
                }
            }
        } elseif (is_string($headersOrBody) || $headersOrBody instanceof Reader) {
            $body = $headersOrBody;
        } elseif ($headersOrBody !== null) {
            throw new \InvalidArgumentException('Second argument must be headers array or body');
        }

        if (is_string($body) || $body instanceof Reader) {
            $this->setBody($body);
        } elseif ($body !== null) {
            throw new \InvalidArgumentException('Body must be string, Reader, or null');
        }
    }

    /**
     * Create a userland Response from a kernel response (client-side).
     *
     * Note: this consumes the kernel response body and turns it into a Reader.
     */
    public static function fromKernelResponse(KernelResponse $kernelResponse): self
    {
        $resp = new self((int)$kernelResponse->status());
        $resp->version = $kernelResponse->version();

        /** @var array<string, list<string>> $headers */
        $headers = $kernelResponse->headers();
        $resp->headers = self::normalizeHeaderMap($headers);

        $kernelReader = $kernelResponse->body();
        if ($kernelReader instanceof \Async\Kernel\IO\AsyncReader) {
            /** @var ReaderWrapper $reader */
            $reader = IO::kernelToWrapper($kernelReader);
            $resp->body = $reader;
        }

        return $resp;
    }

    /**
     * Get HTTP status code
     *
     * @return int
     */
    public function status(): int
    {
        return $this->status;
    }

    /**
     * Get HTTP version (e.g., "HTTP/1.1", "HTTP/2.0")
     *
     * @return string
     */
    public function version(): string
    {
        return $this->version ?? '';
    }

    /**
     * Get response headers
     *
     * @return array<string, list<string>>
     */
    public function headers(): array
    {
        return $this->headers;
    }

    /**
     * Get a specific header value
     *
     * @param string $name Header name (case-insensitive)
     * @return string|null
     */
    public function header(string $name): ?string
    {
        $values = $this->headerValues($name);
        return $values[0] ?? null;
    }

    /**
     * Get all header values for a name (case-insensitive).
     *
     * @return list<string>
     */
    public function headerValues(string $name): array
    {
        $key = strtolower($name);
        return $this->headers[$key] ?? [];
    }

    /**
     * Get header line (comma-joined), similar to PSR-7.
     */
    public function headerLine(string $name): string
    {
        return implode(', ', $this->headerValues($name));
    }

    /**
     * Get response body reader (streaming).
     */
    public function bodyReader(): Reader
    {
        return $this->body ?? self::emptyReader();
    }

    /**
     * Read the whole response body as string (consumes the body).
     */
    public function text(): string
    {
        if ($this->cachedText !== null) {
            return $this->cachedText;
        }

        $this->cachedText = self::readAll($this->bodyReader());
        return $this->cachedText;
    }

    /**
     * Get response body as JSON array (auto-suspends for async read)
     *
     * @return array|null
     */
    public function json(): ?array
    {
        $text = $this->text();
        return json_decode($text, true) ?: null;
    }

    /**
     * Get content length from headers
     *
     * @return int|null
     */
    public function contentLength(): ?int
    {
        $v = $this->header('content-length');
        if ($v === null) {
            return null;
        }
        $n = (int)$v;
        return $n > 0 ? $n : null;
    }

    /**
     * Check if response status is successful (2xx)
     *
     * @return bool
     */
    public function isOk(): bool
    {
        $status = $this->status();
        return $status >= 200 && $status < 300;
    }

    /**
     * Check if response status is a redirect (3xx)
     *
     * @return bool
     */
    public function isRedirect(): bool
    {
        $status = $this->status();
        return $status >= 300 && $status < 400;
    }

    /**
     * Check if response status is a client error (4xx)
     *
     * @return bool
     */
    public function isClientError(): bool
    {
        $status = $this->status();
        return $status >= 400 && $status < 500;
    }

    /**
     * Check if response status is a server error (5xx)
     *
     * @return bool
     */
    public function isServerError(): bool
    {
        $status = $this->status();
        return $status >= 500 && $status < 600;
    }

    /**
     * Read a chunk from the body reader.
     *
     * @return string|null Null when EOF.
     */
    public function readChunk(int $length): ?string
    {
        return $this->bodyReader()->read($length);
    }

    /**
     * Drop the body reader (early terminate).
     */
    public function closeBody(): void
    {
        $this->body = self::emptyReader();
    }

    // Server-side methods for building responses

    /**
     * Set HTTP status code (server-side)
     *
     * @param int $status
     * @return self
     */
    public function withStatus(int $status): self
    {
        $this->status = $status;
        return $this;
    }

    /**
     * Set a response header (server-side)
     *
     * @param string $name
     * @param string $value
     * @return self
     */
    public function withHeader(string $name, string $value): self
    {
        $this->headers[strtolower($name)] = [$value];
        return $this;
    }

    /**
     * Append a header value without overwriting existing ones.
     */
    public function withAddedHeader(string $name, string $value): self
    {
        $key = strtolower($name);
        $this->headers[$key] ??= [];
        $this->headers[$key][] = $value;
        return $this;
    }

    /**
     * Set response body (server-side)
     *
     * @param string|Reader $body
     * @return self
     */
    public function withBody(string|Reader $body): self
    {
        if (is_string($body)) {
            $this->body = self::readerFromString($body);
            $this->cachedText = $body;
            return $this;
        }

        $this->body = $body;
        $this->cachedText = null;
        return $this;
    }

    /**
     * Set JSON response body (server-side)
     *
     * @param array|object $data
     * @return self
     */
    public function withJson(array|object $data): self
    {
        $json = json_encode($data);
        if ($json === false) {
            throw new \InvalidArgumentException('Failed to encode JSON body');
        }
        $this->withHeader('Content-Type', 'application/json');
        $this->withBody($json);
        return $this;
    }

    /**
     * Set HTML response body (server-side)
     *
     * @param string $html
     * @return self
     */
    public function withHtml(string $html): self
    {
        $this->withHeader('Content-Type', 'text/html; charset=utf-8');
        $this->withBody($html);
        return $this;
    }

    /**
     * Set plain text response body (server-side)
     *
     * @param string $text
     * @return self
     */
    public function withText(string $text): self
    {
        $this->withHeader('Content-Type', 'text/plain; charset=utf-8');
        $this->withBody($text);
        return $this;
    }

    /**
     * Convert to kernel response (for returning from server handlers).
     */
    public function toKernelResponse(): KernelResponse
    {
        $kernel = new KernelResponse();
        $kernel->setStatus($this->status);

        foreach ($this->headers as $name => $values) {
            foreach ($values as $i => $value) {
                if ($i === 0) {
                    $kernel->setHeader($name, $value);
                } else {
                    $kernel->appendHeader($name, $value);
                }
            }
        }

        if ($this->body !== null) {
            $kernelAsyncReader = self::toKernelAsyncReader($this->body);
            $kernel->setBody($kernelAsyncReader);
        }

        return $kernel;
    }

    /** @param array<string, list<string>> $headers */
    private static function normalizeHeaderMap(array $headers): array
    {
        $out = [];
        foreach ($headers as $name => $values) {
            $key = strtolower((string)$name);
            $out[$key] = array_values(array_map('strval', $values));
        }
        return $out;
    }

    private static function emptyReader(): Reader
    {
        return self::readerFromString('');
    }

    private static function readerFromString(string $bytes): Reader
    {
        $bytesReader = BytesReader::fromBytes($bytes);
        $kernelReader = $bytesReader->castTo(IO::READ);
        return IO::kernelToWrapper($kernelReader);
    }

    private static function readAll(Reader $reader): string
    {
        $buf = '';
        while (true) {
            $chunk = $reader->read(8192);
            if ($chunk === null || $chunk === '') {
                break;
            }
            $buf .= $chunk;
        }
        return $buf;
    }

    private static function toKernelAsyncReader(Reader $reader): \Async\Kernel\IO\AsyncReader
    {
        // Fast path: if reader implements AsyncIO, use unwrap() for zero-cost conversion
        if ($reader instanceof TokioIO) {
            $kernel = $reader->unwrap();
            if ($kernel instanceof \Async\Kernel\IO\AsyncReader) {
                return $kernel;
            }
            // Fallback to castTo if unwrap doesn't return AsyncReader
            $kernel = $reader->castTo(IO::READ);
            if ($kernel instanceof \Async\Kernel\IO\AsyncReader) {
                return $kernel;
            }
        }

        // Slow path: wrap as PHP IO and cast (fallback for arbitrary PHP IO objects)
        $phpReader = IO::wrapPhpIo($reader, IO::READ);
        $kernel = $phpReader->castTo(IO::READ);

        if (!$kernel instanceof \Async\Kernel\IO\AsyncReader) {
            throw new \RuntimeException('Failed to convert Reader to kernel AsyncReader');
        }

        return $kernel;
    }

    private static function reasonPhraseFor(int $status): string
    {
        return match ($status) {
            200 => 'OK',
            201 => 'Created',
            202 => 'Accepted',
            204 => 'No Content',
            301 => 'Moved Permanently',
            302 => 'Found',
            303 => 'See Other',
            307 => 'Temporary Redirect',
            308 => 'Permanent Redirect',
            400 => 'Bad Request',
            401 => 'Unauthorized',
            403 => 'Forbidden',
            404 => 'Not Found',
            409 => 'Conflict',
            429 => 'Too Many Requests',
            500 => 'Internal Server Error',
            502 => 'Bad Gateway',
            503 => 'Service Unavailable',
            default => '',
        };
    }
}
