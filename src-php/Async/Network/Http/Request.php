<?php

namespace Async\Network\Http;

use Async\IO;
use Async\IO\Reader;
use Async\Kernel\IO\BytesReader;
use Async\Kernel\Network\Http\HttpRequest as KernelRequest;
use Async\IO\Wrapper\ReaderWrapper;

/**
 * HTTP Request (userland) - kernel-agnostic.
 *
 * - Does not hold Async\Kernel\Network\Http\HttpRequest directly.
 * - Convert via Request::fromKernelRequest() and Request->toKernelRequest().
 * - Body is represented as Async\IO\Reader for streaming.
 *
 * Usage (Client):
 * ```php
 * $request = new Request('POST', 'https://api.example.com/users');
 * $request->header('Authorization', 'Bearer token')
 *         ->bodyJson(['name' => 'John']);
 * $response = $client->send($request);
 * ```
 *
 * Usage (Server):
 * ```php
 * function handler(Request $req): Response {
 *     $method = $req->method();
 *     $path = $req->path();
 *     $body = $req->body();
 *     // ...
 * }
 * ```
 */
class Request
{
    private string $method;
    private string $uri;
    private ?string $version = null;

    /** @var array<string, list<string>> */
    private array $headers = [];

    private ?Reader $body = null;
    private ?string $cachedText = null;
    private ?float $timeoutSeconds = null;

    /**
     * Create a Request (client-side).
     */
    public function __construct(string $method, string $uri)
    {
        $this->method = strtoupper($method);
        $this->uri = $uri;
    }

    /**
     * Create a userland Request from a kernel request (server-side).
     */
    public static function fromKernelRequest(KernelRequest $kernelRequest): self
    {
        $req = new self($kernelRequest->method(), $kernelRequest->uri());
        $req->version = $kernelRequest->version();

        /** @var array<string, list<string>> $headers */
        $headers = $kernelRequest->headers();
        $req->headers = self::normalizeHeaderMap($headers);

        if (method_exists($kernelRequest, 'body')) {
            $kernelReader = $kernelRequest->body();
            if ($kernelReader instanceof \Async\Kernel\IO\AsyncReader) {
                /** @var ReaderWrapper $reader */
                $reader = IO::kernelToWrapper($kernelReader);
                $req->body = $reader;
            }
        }

        return $req;
    }

    /**
     * Get HTTP method (GET, POST, etc.)
     *
     * @return string
     */
    public function method(): string
    {
        return $this->method;
    }

    /**
     * Get full URI
     *
     * @return string
     */
    public function uri(): string
    {
        return $this->uri;
    }

    /**
     * Get request path (e.g., "/api/users")
     *
     * @return string
     */
    public function path(): string
    {
        $path = parse_url($this->uri, PHP_URL_PATH);
        return $path !== null && $path !== '' ? $path : '/';
    }

    /**
     * Get HTTP version
     *
     * @return string
     */
    public function version(): string
    {
        return $this->version ?? '';
    }

    /**
     * Get all request headers
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
     * @return string|null First value if exists
     */
    public function getHeader(string $name): ?string
    {
        $values = $this->getHeaderValues($name);
        return $values[0] ?? null;
    }

    /**
     * Get all header values for a name (case-insensitive).
     *
     * @return list<string>
     */
    public function getHeaderValues(string $name): array
    {
        $key = strtolower($name);
        return $this->headers[$key] ?? [];
    }

    /**
     * Get header line (comma-joined), similar to PSR-7.
     */
    public function getHeaderLine(string $name): string
    {
        $values = $this->getHeaderValues($name);
        return implode(', ', $values);
    }

    /**
     * Get request body reader (streaming).
     */
    public function bodyReader(): Reader
    {
        return $this->body ?? self::emptyReader();
    }

    /**
     * Read the whole request body as string (consumes the body).
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
     * Get request body as string (consumes the body).
     */
    public function body(): string
    {
        return $this->text();
    }

    /**
     * Get request body as JSON array
     *
     * @return array|null
     */
    public function json(): ?array
    {
        $body = $this->body();
        return json_decode($body, true) ?: null;
    }

    /**
     * Get query parameters
     *
     * @return array<string, string>
     */
    public function query(): array
    {
        $queryString = parse_url($this->uri, PHP_URL_QUERY);

        if ($queryString === null) {
            return [];
        }

        parse_str($queryString, $params);
        return $params;
    }

    /**
     * Get a specific query parameter
     *
     * @param string $name
     * @param string|null $default
     * @return string|null
     */
    public function getQuery(string $name, ?string $default = null): ?string
    {
        $query = $this->query();
        return $query[$name] ?? $default;
    }

    /**
     * Get raw query string (without "?").
     */
    public function queryString(): string
    {
        return (string)(parse_url($this->uri, PHP_URL_QUERY) ?? '');
    }

    // Client-side builder methods

    /**
     * Set a request header (client-side builder)
     *
     * @param string $name
     * @param string $value
     * @return self
     */
    public function header(string $name, string $value): self
    {
        $this->headers[strtolower($name)] = [$value];
        return $this;
    }

    /**
     * Append a header value without overwriting existing ones.
     */
    public function appendHeader(string $name, string $value): self
    {
        $key = strtolower($name);
        $this->headers[$key] ??= [];
        $this->headers[$key][] = $value;
        return $this;
    }

    /**
     * Set request body as text (client-side builder)
     *
     * @param string $body
     * @param string|null $contentType
     * @return self
     */
    public function bodyText(string $body, ?string $contentType = null): self
    {
        if ($contentType !== null) {
            $this->header('Content-Type', $contentType);
        }

        $this->body = self::readerFromString($body);
        $this->cachedText = $body;
        return $this;
    }

    /**
     * Set request body as JSON (client-side builder)
     *
     * @param array|object $data
     * @return self
     */
    public function bodyJson(array|object $data): self
    {
        $json = json_encode($data);
        if ($json === false) {
            throw new \InvalidArgumentException('Failed to encode JSON body');
        }

        $this->header('Content-Type', 'application/json');
        $this->body = self::readerFromString($json);
        $this->cachedText = $json;
        return $this;
    }

    /**
     * Set request body as form data (client-side builder)
     *
     * @param array<string, string> $data
     * @return self
     */
    public function bodyForm(array $data): self
    {
        $body = http_build_query($data);
        $this->header('Content-Type', 'application/x-www-form-urlencoded');
        $this->body = self::readerFromString($body);
        $this->cachedText = $body;
        return $this;
    }

    /**
     * Set request body as a Reader (streaming).
     */
    public function bodyStream(Reader $reader): self
    {
        $this->body = $reader;
        $this->cachedText = null;
        return $this;
    }

    /**
     * Set timeout for this request (client-side builder)
     *
     * @param float $seconds
     * @return self
     */
    public function timeout(float $seconds): self
    {
        $this->timeoutSeconds = $seconds;
        return $this;
    }

    /**
     * Convert to kernel request (for sending via kernel client/server).
     */
    public function toKernelRequest(): KernelRequest
    {
        $kernel = new KernelRequest($this->method, $this->uri);

        if ($this->timeoutSeconds !== null) {
            $kernel->setTimeout($this->timeoutSeconds);
        }

        foreach ($this->headers as $name => $values) {
            foreach ($values as $i => $value) {
                if ($i === 0) {
                    $kernel->setHeader($name, $value);
                    continue;
                }
                if (method_exists($kernel, 'appendHeader')) {
                    $kernel->appendHeader($name, $value);
                } else {
                    $kernel->setHeader($name, $value);
                }
            }
        }

        if ($this->body !== null) {
            $kernelAsyncReader = self::toKernelAsyncReader($this->body);
            if (method_exists($kernel, 'bodyStream')) {
                $kernel->bodyStream($kernelAsyncReader);
            } elseif (method_exists($kernel, 'setBody')) {
                $kernel->setBody($kernelAsyncReader);
            }
        }

        return $kernel;
    }

    /**
     * Backward-compatible alias.
     *
     * @internal
     */
    public function getKernel(): KernelRequest
    {
        return $this->toKernelRequest();
    }

    /**
     * Backward-compatible alias.
     */
    public function getBody(): Reader
    {
        return $this->bodyReader();
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
        $phpReader = IO::wrapPhpIo($reader, IO::READ);
        $kernelReader = $phpReader->castTo(IO::READ);

        if (!$kernelReader instanceof \Async\Kernel\IO\AsyncReader) {
            throw new \RuntimeException('Failed to convert Reader to kernel AsyncReader');
        }

        return $kernelReader;
    }
}
