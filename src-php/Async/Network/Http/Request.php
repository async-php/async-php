<?php

namespace Async\Network\Http;

use Async\Kernel\Network\Http\HttpRequest as KernelRequest;
use Async\Kernel\Network\Http\HttpResponseBody;
use Fiber;

class Request
{
    protected KernelRequest $kernel;
    private ?RequestBody $body = null;

    public function __construct(KernelRequest $kernel)
    {
        $this->kernel = $kernel;
    }

    /**
     * Get the HTTP method (GET, POST, etc.)
     */
    public function getMethod(): string
    {
        return $this->kernel->get_method();
    }

    /**
     * Get the request URI
     */
    public function getUri(): string
    {
        return $this->kernel->get_uri();
    }

    /**
     * Get the HTTP version (e.g., "1.1", "2.0", "3.0")
     */
    public function getVersion(): string
    {
        return $this->kernel->get_version();
    }

    /**
     * Get a specific header value
     */
    public function getHeader(string $name): ?string
    {
        return $this->kernel->get_header($name);
    }

    /**
     * Get all headers as array
     */
    public function getHeaders(): array
    {
        return $this->kernel->get_headers();
    }

    /**
     * Get the request body
     *
     * Returns a RequestBody object for streaming large bodies,
     * or null if no body is present
     */
    public function getBody(): ?RequestBody
    {
        if ($this->body === null) {
            $bodyData = $this->kernel->get_body();

            // If body is a string, wrap it in RequestBody
            if (is_string($bodyData)) {
                $this->body = RequestBody::fromString($bodyData);
            }
            // If body is HttpResponseBody (streaming), wrap it
            elseif ($bodyData instanceof HttpResponseBody) {
                $this->body = new RequestBody($bodyData);
            }
        }

        return $this->body;
    }

    /**
     * Get the entire body content as a string
     *
     * Warning: This loads the entire body into memory
     */
    public function getBodyAsString(): string
    {
        $body = $this->getBody();
        if ($body === null) {
            return '';
        }

        return $body->readAll();
    }

    /**
     * Parse the body as JSON
     */
    public function getBodyAsJson(bool $assoc = true): mixed
    {
        $content = $this->getBodyAsString();
        if (empty($content)) {
            return $assoc ? [] : null;
        }

        $result = json_decode($content, $assoc);
        if (json_last_error() !== JSON_ERROR_NONE) {
            throw new \RuntimeException('Failed to decode JSON: ' . json_last_error_msg());
        }

        return $result;
    }

    /**
     * Check if request is using HTTPS
     */
    public function isSecure(): bool
    {
        $version = $this->getVersion();
        return in_array($version, ['2.0', '3.0'], true) ||
               $this->getHeader('X-Forwarded-Proto') === 'https';
    }

    /**
     * Get the underlying kernel request
     */
    public function getKernel(): KernelRequest
    {
        return $this->kernel;
    }
}
