<?php

namespace Async\Network\Http;

use Async\Kernel\Network\Http\HttpRequest as KernelRequest;

/**
 * HTTP Request - Wraps kernel HttpRequest and hides async Future details
 *
 * This class provides a simple interface for both server and client requests,
 * automatically handling async operations.
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
    private KernelRequest $kernel;
    private ?string $cachedBody = null;

    /**
     * Create a Request
     *
     * @param string|KernelRequest $methodOrKernel HTTP method (GET, POST, etc.) or kernel request
     * @param string|null $url URL (required if first param is method)
     */
    public function __construct(string|KernelRequest $methodOrKernel, ?string $url = null)
    {
        if ($methodOrKernel instanceof KernelRequest) {
            $this->kernel = $methodOrKernel;
        } else {
            if ($url === null) {
                throw new \InvalidArgumentException('URL is required when creating request with method');
            }
            $this->kernel = new KernelRequest($methodOrKernel, $url);
        }
    }

    /**
     * Get HTTP method (GET, POST, etc.)
     *
     * @return string
     */
    public function method(): string
    {
        return $this->kernel->method();
    }

    /**
     * Get full URI
     *
     * @return string
     */
    public function uri(): string
    {
        return $this->kernel->uri();
    }

    /**
     * Get request path (e.g., "/api/users")
     *
     * @return string
     */
    public function path(): string
    {
        return $this->kernel->path();
    }

    /**
     * Get HTTP version
     *
     * @return string
     */
    public function version(): string
    {
        return $this->kernel->version();
    }

    /**
     * Get all request headers
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
    public function getHeader(string $name): ?string
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
     * Get request body as string
     *
     * @return string
     */
    public function body(): string
    {
        if ($this->cachedBody !== null) {
            return $this->cachedBody;
        }

        $this->cachedBody = $this->kernel->body();
        return $this->cachedBody;
    }

    /**
     * Get request body as JSON array
     *
     * @return array|null
     */
    public function json(): ?array
    {
        $body = $this->body();
        return json_decode($body, true);
    }

    /**
     * Get query parameters
     *
     * @return array<string, string>
     */
    public function query(): array
    {
        $uri = $this->uri();
        $queryString = parse_url($uri, PHP_URL_QUERY);

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
        $this->kernel->header($name, $value);
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
        $this->kernel->bodyText($body, $contentType);
        $this->cachedBody = $body;
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
        $this->kernel->bodyJson($json);
        $this->cachedBody = $json;
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
        $this->kernel->bodyText($body, 'application/x-www-form-urlencoded');
        $this->cachedBody = $body;
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
        $this->kernel->timeout($seconds);
        return $this;
    }

    /**
     * Get the underlying kernel request
     *
     * @internal
     * @return KernelRequest
     */
    public function getKernel(): KernelRequest
    {
        return $this->kernel;
    }
}
