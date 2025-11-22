<?php

namespace Async\Network\Http;

use Async\Kernel\Network\Http\HttpRequest as KernelRequest;
use Psr\Http\Message\ServerRequestInterface;
use Psr\Http\Message\StreamInterface;
use Psr\Http\Message\UriInterface;

/**
 * PSR-7 ServerRequest adapter for kernel HttpRequest
 *
 * Wraps the low-level KernelHttpRequest to provide PSR-7 ServerRequestInterface.
 * This adapter is immutable - with*() methods return new instances.
 */
class Psr7ServerRequest implements ServerRequestInterface
{
    private KernelRequest $kernelRequest;
    private array $serverParams;
    private array $cookieParams;
    private array $queryParams;
    private array $uploadedFiles;
    private ?array $parsedBody;
    private array $attributes;
    private ?UriInterface $uri = null;
    private ?StreamInterface $body = null;
    private array $headers; // Normalized header names => array of values

    /**
     * Create PSR-7 server request from kernel HttpRequest
     */
    public function __construct(KernelRequest $kernelRequest)
    {
        $this->kernelRequest = $kernelRequest;

        // Convert kernel headers to PSR-7 format
        $kernelHeaders = $kernelRequest->headers();
        $this->headers = [];
        foreach ($kernelHeaders as $name => $value) {
            $this->headers[strtolower($name)] = [$value];
        }

        // Parse query parameters from URI
        $uriString = $kernelRequest->uri();
        $this->queryParams = [];
        if (($pos = strpos($uriString, '?')) !== false) {
            parse_str(substr($uriString, $pos + 1), $this->queryParams);
        }

        // Parse cookies from Cookie header
        $this->cookieParams = [];
        if (isset($this->headers['cookie'])) {
            $cookieHeader = $this->headers['cookie'][0];
            $cookies = explode(';', $cookieHeader);
            foreach ($cookies as $cookie) {
                $parts = explode('=', trim($cookie), 2);
                if (count($parts) === 2) {
                    $this->cookieParams[$parts[0]] = $parts[1];
                }
            }
        }

        $this->serverParams = [];
        $this->uploadedFiles = [];
        $this->parsedBody = null;
        $this->attributes = [];
    }

    // PSR-7 ServerRequestInterface methods

    public function getServerParams(): array
    {
        return $this->serverParams;
    }

    public function getCookieParams(): array
    {
        return $this->cookieParams;
    }

    public function withCookieParams(array $cookies): ServerRequestInterface
    {
        $new = clone $this;
        $new->cookieParams = $cookies;
        return $new;
    }

    public function getQueryParams(): array
    {
        return $this->queryParams;
    }

    public function withQueryParams(array $query): ServerRequestInterface
    {
        $new = clone $this;
        $new->queryParams = $query;
        return $new;
    }

    public function getUploadedFiles(): array
    {
        return $this->uploadedFiles;
    }

    public function withUploadedFiles(array $uploadedFiles): ServerRequestInterface
    {
        $new = clone $this;
        $new->uploadedFiles = $uploadedFiles;
        return $new;
    }

    public function getParsedBody()
    {
        if ($this->parsedBody === null) {
            // Auto-parse based on Content-Type
            $contentType = $this->getHeaderLine('Content-Type');
            $body = (string)$this->getBody();

            if (str_contains($contentType, 'application/json')) {
                $this->parsedBody = json_decode($body, true) ?? [];
            } elseif (str_contains($contentType, 'application/x-www-form-urlencoded')) {
                parse_str($body, $this->parsedBody);
            } elseif (str_contains($contentType, 'multipart/form-data')) {
                // Multipart parsing is complex, leave as null for now
                $this->parsedBody = [];
            } else {
                $this->parsedBody = [];
            }
        }

        return $this->parsedBody;
    }

    public function withParsedBody($data): ServerRequestInterface
    {
        $new = clone $this;
        $new->parsedBody = $data;
        return $new;
    }

    public function getAttributes(): array
    {
        return $this->attributes;
    }

    public function getAttribute($name, $default = null)
    {
        return $this->attributes[$name] ?? $default;
    }

    public function withAttribute($name, $value): ServerRequestInterface
    {
        $new = clone $this;
        $new->attributes[$name] = $value;
        return $new;
    }

    public function withoutAttribute($name): ServerRequestInterface
    {
        $new = clone $this;
        unset($new->attributes[$name]);
        return $new;
    }

    // PSR-7 RequestInterface methods

    public function getRequestTarget(): string
    {
        $target = $this->kernelRequest->uri();

        // Remove scheme and authority if present
        if (preg_match('#^https?://[^/]+(.*)$#', $target, $matches)) {
            $target = $matches[1];
        }

        return $target ?: '/';
    }

    public function withRequestTarget($requestTarget): ServerRequestInterface
    {
        // Cannot modify kernel request, would need to store override
        throw new \RuntimeException('withRequestTarget is not supported on kernel request adapter');
    }

    public function getMethod(): string
    {
        return strtoupper($this->kernelRequest->method());
    }

    public function withMethod($method): ServerRequestInterface
    {
        throw new \RuntimeException('withMethod is not supported on kernel request adapter');
    }

    public function getUri(): UriInterface
    {
        if ($this->uri === null) {
            $this->uri = new Uri($this->kernelRequest->uri());
        }
        return $this->uri;
    }

    public function withUri(UriInterface $uri, $preserveHost = false): ServerRequestInterface
    {
        $new = clone $this;
        $new->uri = $uri;

        if (!$preserveHost || !$this->hasHeader('Host')) {
            if ($uri->getHost() !== '') {
                $host = $uri->getHost();
                if ($uri->getPort() !== null) {
                    $host .= ':' . $uri->getPort();
                }
                $new->headers['host'] = [$host];
            }
        }

        return $new;
    }

    // PSR-7 MessageInterface methods

    public function getProtocolVersion(): string
    {
        $version = $this->kernelRequest->version();
        // Parse "HTTP/1.1" -> "1.1"
        if (preg_match('/HTTP\/(\d+\.\d+)/', $version, $matches)) {
            return $matches[1];
        }
        return '1.1';
    }

    public function withProtocolVersion($version): ServerRequestInterface
    {
        throw new \RuntimeException('withProtocolVersion is not supported on kernel request adapter');
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

    public function withHeader($name, $value): ServerRequestInterface
    {
        $normalized = strtolower($name);
        $new = clone $this;
        $new->headers[$normalized] = is_array($value) ? $value : [$value];
        return $new;
    }

    public function withAddedHeader($name, $value): ServerRequestInterface
    {
        $normalized = strtolower($name);
        $new = clone $this;
        $new->headers[$normalized] = array_merge(
            $this->headers[$normalized] ?? [],
            is_array($value) ? $value : [$value]
        );
        return $new;
    }

    public function withoutHeader($name): ServerRequestInterface
    {
        $normalized = strtolower($name);
        $new = clone $this;
        unset($new->headers[$normalized]);
        return $new;
    }

    public function getBody(): StreamInterface
    {
        if ($this->body === null) {
            $bodyContent = $this->kernelRequest->body();
            $this->body = new StringStream($bodyContent);
        }
        return $this->body;
    }

    public function withBody(StreamInterface $body): ServerRequestInterface
    {
        $new = clone $this;
        $new->body = $body;
        return $new;
    }

    /**
     * Get the underlying kernel request for advanced usage
     */
    public function getKernelRequest(): KernelRequest
    {
        return $this->kernelRequest;
    }
}
