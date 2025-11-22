<?php

namespace Async\Network\Http;

use Async\Kernel\Network\Http\HttpResponse as KernelResponse;
use Psr\Http\Message\ResponseInterface;
use Psr\Http\Message\StreamInterface;

/**
 * PSR-7 HTTP Response implementation for HTTP client
 *
 * Wraps the kernel HttpResponse from reqwest-based client to provide
 * PSR-7 ResponseInterface compatibility with proper immutability.
 */
class Psr7Response implements ResponseInterface
{
    private KernelResponse $kernelResponse;
    private int $statusCode;
    private string $reasonPhrase;
    private string $protocolVersion;
    private array $headers; // Normalized header names => array of values
    private ?StreamInterface $body;

    /**
     * HTTP status code reason phrases
     */
    private const REASON_PHRASES = [
        100 => 'Continue', 101 => 'Switching Protocols',
        200 => 'OK', 201 => 'Created', 202 => 'Accepted', 203 => 'Non-Authoritative Information',
        204 => 'No Content', 205 => 'Reset Content', 206 => 'Partial Content',
        300 => 'Multiple Choices', 301 => 'Moved Permanently', 302 => 'Found', 303 => 'See Other',
        304 => 'Not Modified', 307 => 'Temporary Redirect', 308 => 'Permanent Redirect',
        400 => 'Bad Request', 401 => 'Unauthorized', 402 => 'Payment Required', 403 => 'Forbidden',
        404 => 'Not Found', 405 => 'Method Not Allowed', 406 => 'Not Acceptable',
        407 => 'Proxy Authentication Required', 408 => 'Request Timeout', 409 => 'Conflict',
        410 => 'Gone', 411 => 'Length Required', 412 => 'Precondition Failed',
        413 => 'Payload Too Large', 414 => 'URI Too Long', 415 => 'Unsupported Media Type',
        416 => 'Range Not Satisfiable', 417 => 'Expectation Failed', 426 => 'Upgrade Required',
        500 => 'Internal Server Error', 501 => 'Not Implemented', 502 => 'Bad Gateway',
        503 => 'Service Unavailable', 504 => 'Gateway Timeout', 505 => 'HTTP Version Not Supported',
    ];

    /**
     * Create PSR-7 response from kernel HttpResponse
     *
     * @param KernelResponse $kernelResponse The kernel response from HTTP client
     * @param int|null $statusCode Optional override for status code
     * @param string|null $reasonPhrase Optional override for reason phrase
     * @param string|null $protocolVersion Optional override for protocol version
     * @param array|null $headers Optional override for headers
     * @param StreamInterface|null $body Optional override for body stream
     */
    public function __construct(
        KernelResponse $kernelResponse,
        ?int $statusCode = null,
        ?string $reasonPhrase = null,
        ?string $protocolVersion = null,
        ?array $headers = null,
        ?StreamInterface $body = null
    ) {
        $this->kernelResponse = $kernelResponse;
        $this->statusCode = $statusCode ?? $kernelResponse->status();
        $this->reasonPhrase = $reasonPhrase ?? (self::REASON_PHRASES[$this->statusCode] ?? '');

        // Parse protocol version from kernel (e.g., "HTTP/1.1" -> "1.1")
        $kernelVersion = $kernelResponse->version();
        $this->protocolVersion = $protocolVersion ?? (
            preg_match('/HTTP\/(\d+\.\d+)/', $kernelVersion, $matches) ? $matches[1] : '1.1'
        );

        // Convert kernel headers to PSR-7 format (array of arrays)
        if ($headers === null) {
            $kernelHeaders = $kernelResponse->headers();
            $this->headers = [];
            foreach ($kernelHeaders as $name => $value) {
                $this->headers[strtolower($name)] = [$value];
            }
        } else {
            $this->headers = $headers;
        }

        $this->body = $body;
    }

    // PSR-7 ResponseInterface methods

    public function getStatusCode(): int
    {
        return $this->statusCode;
    }

    public function withStatus($code, $reasonPhrase = ''): ResponseInterface
    {
        return new self(
            $this->kernelResponse,
            $code,
            $reasonPhrase ?: (self::REASON_PHRASES[$code] ?? ''),
            $this->protocolVersion,
            $this->headers,
            $this->body
        );
    }

    public function getReasonPhrase(): string
    {
        return $this->reasonPhrase;
    }

    // PSR-7 MessageInterface methods

    public function getProtocolVersion(): string
    {
        return $this->protocolVersion;
    }

    public function withProtocolVersion($version): ResponseInterface
    {
        return new self(
            $this->kernelResponse,
            $this->statusCode,
            $this->reasonPhrase,
            $version,
            $this->headers,
            $this->body
        );
    }

    public function getHeaders(): array
    {
        return $this->headers;
    }

    public function hasHeader($name): bool
    {
        return isset($this->headers[strtolower($name)]);
    }

    public function getHeader($name): array
    {
        $name = strtolower($name);
        return $this->headers[$name] ?? [];
    }

    public function getHeaderLine($name): string
    {
        return implode(', ', $this->getHeader($name));
    }

    public function withHeader($name, $value): ResponseInterface
    {
        $normalized = strtolower($name);
        $newHeaders = $this->headers;
        $newHeaders[$normalized] = is_array($value) ? $value : [$value];

        return new self(
            $this->kernelResponse,
            $this->statusCode,
            $this->reasonPhrase,
            $this->protocolVersion,
            $newHeaders,
            $this->body
        );
    }

    public function withAddedHeader($name, $value): ResponseInterface
    {
        $normalized = strtolower($name);
        $newHeaders = $this->headers;
        $newHeaders[$normalized] = array_merge(
            $this->headers[$normalized] ?? [],
            is_array($value) ? $value : [$value]
        );

        return new self(
            $this->kernelResponse,
            $this->statusCode,
            $this->reasonPhrase,
            $this->protocolVersion,
            $newHeaders,
            $this->body
        );
    }

    public function withoutHeader($name): ResponseInterface
    {
        $normalized = strtolower($name);
        $newHeaders = $this->headers;
        unset($newHeaders[$normalized]);

        return new self(
            $this->kernelResponse,
            $this->statusCode,
            $this->reasonPhrase,
            $this->protocolVersion,
            $newHeaders,
            $this->body
        );
    }

    public function getBody(): StreamInterface
    {
        if ($this->body === null) {
            // Create stream from kernel response on first access
            try {
                $asyncReader = $this->kernelResponse->stream();
                $contentLength = $this->kernelResponse->contentLength();
                $this->body = new ReaderStream($asyncReader, $contentLength);
            } catch (\Throwable $e) {
                // If stream() fails, create an empty stream by creating a dummy AsyncReader
                // For now, we'll throw the error as this shouldn't happen in normal cases
                throw new \RuntimeException('Failed to create response body stream: ' . $e->getMessage(), 0, $e);
            }
        }
        return $this->body;
    }

    public function withBody(StreamInterface $body): ResponseInterface
    {
        return new self(
            $this->kernelResponse,
            $this->statusCode,
            $this->reasonPhrase,
            $this->protocolVersion,
            $this->headers,
            $body
        );
    }

    /**
     * Get the underlying kernel response for advanced usage
     *
     * @return KernelResponse
     */
    public function getKernelResponse(): KernelResponse
    {
        return $this->kernelResponse;
    }
}
