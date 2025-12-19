<?php

namespace Async\Network\Http;

use Async\Kernel\Network\Http\HttpResponse as KernelResponse;
use Fiber;

/**
 * HTTP Response - Wraps kernel HttpResponse and hides async Future details
 *
 * This class provides a simple interface for both server and client responses,
 * automatically handling Fiber suspension for async operations.
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
    private KernelResponse $kernel;
    private ?string $cachedBody = null;

    /**
     * Create a Response
     *
     * @param KernelResponse|null $kernelResponse Optional kernel response (for client responses)
     */
    public function __construct(?KernelResponse $kernelResponse = null)
    {
        $this->kernel = $kernelResponse ?? new KernelResponse();
    }

    /**
     * Get HTTP status code
     *
     * @return int
     */
    public function status(): int
    {
        return $this->kernel->status();
    }

    /**
     * Get HTTP version (e.g., "HTTP/1.1", "HTTP/2.0")
     *
     * @return string
     */
    public function version(): string
    {
        return $this->kernel->version();
    }

    /**
     * Get response headers
     *
     * @return array<string, string>
     */
    public function headers(): array
    {
        return $this->kernel->headers();
    }

    /**
     * Get a specific header value
     *
     * @param string $name Header name (case-insensitive)
     * @return string|null
     */
    public function header(string $name): ?string
    {
        $headers = $this->headers();
        $name = strtolower($name);

        foreach ($headers as $key => $value) {
            if (strtolower($key) === $name) {
                return $value;
            }
        }

        return null;
    }

    /**
     * Get response body as text (auto-suspends for async read)
     *
     * @return string
     */
    public function text(): string
    {
        if ($this->cachedBody !== null) {
            return $this->cachedBody;
        }

        $this->cachedBody = $this->kernel->text();
        return $this->cachedBody;
    }

    /**
     * Get response body as JSON array (auto-suspends for async read)
     *
     * @return array|null
     */
    public function json(): ?array
    {
        $text = $this->text();
        return json_decode($text, true);
    }

    /**
     * Get content length from headers
     *
     * @return int|null
     */
    public function contentLength(): ?int
    {
        return $this->kernel->contentLength();
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

    // Server-side methods for building responses

    /**
     * Set HTTP status code (server-side)
     *
     * @param int $status
     * @return self
     */
    public function setStatus(int $status): self
    {
        $this->kernel->setStatus($status);
        return $this;
    }

    /**
     * Set a response header (server-side)
     *
     * @param string $name
     * @param string $value
     * @return self
     */
    public function setHeader(string $name, string $value): self
    {
        $this->kernel->setHeader($name, $value);
        return $this;
    }

    /**
     * Set response body (server-side)
     *
     * @param string $body
     * @return self
     */
    public function setBody(string $body): self
    {
        $this->kernel->setBody($body);
        $this->cachedBody = $body;
        return $this;
    }

    /**
     * Set JSON response body (server-side)
     *
     * @param array|object $data
     * @return self
     */
    public function setJson(array|object $data): self
    {
        $json = json_encode($data);
        $this->setHeader('Content-Type', 'application/json');
        $this->setBody($json);
        return $this;
    }

    /**
     * Set HTML response body (server-side)
     *
     * @param string $html
     * @return self
     */
    public function setHtml(string $html): self
    {
        $this->setHeader('Content-Type', 'text/html; charset=utf-8');
        $this->setBody($html);
        return $this;
    }

    /**
     * Set plain text response body (server-side)
     *
     * @param string $text
     * @return self
     */
    public function setText(string $text): self
    {
        $this->setHeader('Content-Type', 'text/plain; charset=utf-8');
        $this->setBody($text);
        return $this;
    }

    /**
     * Get the underlying kernel response
     *
     * @internal
     * @return KernelResponse
     */
    public function getKernel(): KernelResponse
    {
        return $this->kernel;
    }
}
