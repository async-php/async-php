<?php

namespace Async\Network\Http;

use Async\Kernel\Network\Http\HttpResponse as KernelResponse;
use Async\Kernel\Network\Http\HttpResponseBody;
use Fiber;

class ClientResponse
{
    private KernelResponse $kernel;
    private ?string $bodyCache = null;

    public function __construct(KernelResponse $kernel)
    {
        $this->kernel = $kernel;
    }

    /**
     * Get the HTTP status code
     */
    public function getStatusCode(): int
    {
        return $this->kernel->getStatusCode();
    }

    /**
     * Get the reason phrase (e.g., "OK", "Not Found")
     */
    public function getReasonPhrase(): string
    {
        return $this->kernel->getReasonPhrase();
    }

    /**
     * Get the HTTP version (e.g., "1.1", "2.0")
     */
    public function getVersion(): string
    {
        return $this->kernel->getVersion();
    }

    /**
     * Get a specific header value
     */
    public function getHeader(string $name): ?string
    {
        return $this->kernel->getHeader($name);
    }

    /**
     * Get all headers as an associative array
     */
    public function getHeaders(): array
    {
        return $this->kernel->getHeaders();
    }

    /**
     * Check if the response has a specific header
     */
    public function hasHeader(string $name): bool
    {
        return $this->getHeader($name) !== null;
    }

    /**
     * Get the response body as a string
     *
     * The body is cached after the first read
     */
    public function getBody(): string
    {
        if ($this->bodyCache !== null) {
            return $this->bodyCache;
        }

        $bodyObject = $this->kernel->getBody();

        if ($bodyObject === null) {
            $this->bodyCache = '';
            return '';
        }

        if ($bodyObject instanceof HttpResponseBody) {
            $future = $bodyObject->readAll();
            $content = Fiber::suspend($future);
            $this->bodyCache = $content ?? '';
            return $this->bodyCache;
        }

        // Fallback for string body
        $this->bodyCache = (string)$bodyObject;
        return $this->bodyCache;
    }

    /**
     * Get the response body as JSON
     *
     * @param bool $assoc When true, returns array; when false, returns object
     * @return mixed
     * @throws \RuntimeException if JSON decoding fails
     */
    public function json(bool $assoc = true): mixed
    {
        $content = $this->getBody();

        if (empty($content)) {
            return $assoc ? [] : null;
        }

        $result = json_decode($content, $assoc);

        if (json_last_error() !== JSON_ERROR_NONE) {
            throw new \RuntimeException(
                'Failed to decode JSON: ' . json_last_error_msg()
            );
        }

        return $result;
    }

    /**
     * Get the underlying kernel response object
     */
    public function getKernel(): KernelResponse
    {
        return $this->kernel;
    }

    /**
     * Check if the response is successful (2xx)
     */
    public function isSuccess(): bool
    {
        return $this->kernel->isSuccess();
    }

    /**
     * Check if the response is a redirect (3xx)
     */
    public function isRedirect(): bool
    {
        return $this->kernel->isRedirect();
    }

    /**
     * Check if the response is a client error (4xx)
     */
    public function isClientError(): bool
    {
        return $this->kernel->isClientError();
    }

    /**
     * Check if the response is a server error (5xx)
     */
    public function isServerError(): bool
    {
        return $this->kernel->isServerError();
    }

    /**
     * Get the content type from the Content-Type header
     */
    public function getContentType(): ?string
    {
        $contentType = $this->getHeader('content-type');

        if ($contentType === null) {
            return null;
        }

        // Extract just the MIME type (remove charset, etc.)
        $parts = explode(';', $contentType, 2);
        return trim($parts[0]);
    }

    /**
     * Check if the response content type is JSON
     */
    public function isJson(): bool
    {
        $contentType = $this->getContentType();
        return $contentType !== null &&
               (str_contains($contentType, 'json') ||
                str_contains($contentType, 'javascript'));
    }

    /**
     * Check if the response content type is HTML
     */
    public function isHtml(): bool
    {
        $contentType = $this->getContentType();
        return $contentType !== null && str_contains($contentType, 'html');
    }

    /**
     * Check if the response content type is XML
     */
    public function isXml(): bool
    {
        $contentType = $this->getContentType();
        return $contentType !== null && str_contains($contentType, 'xml');
    }

    /**
     * Get the response body length from Content-Length header
     */
    public function getContentLength(): ?int
    {
        $length = $this->getHeader('content-length');
        return $length !== null ? (int)$length : null;
    }

    /**
     * Magic method to convert response to string
     */
    public function __toString(): string
    {
        return $this->getBody();
    }
}
