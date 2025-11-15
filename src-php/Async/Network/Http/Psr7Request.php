<?php

namespace Async\Network\Http;

use Psr\Http\Message\RequestInterface;
use Psr\Http\Message\StreamInterface;
use Psr\Http\Message\UriInterface;

/**
 * PSR-7 HTTP Request implementation
 *
 * Represents an outgoing HTTP request with immutability.
 */
class Psr7Request implements RequestInterface
{
    private string $method;
    private UriInterface $uri;
    private string $protocolVersion = '1.1';
    private array $headers = []; // Normalized header names => array of values
    private ?StreamInterface $body;
    private string $requestTarget = '';

    /**
     * Create a new HTTP request
     *
     * @param string $method HTTP method (GET, POST, etc.)
     * @param string|UriInterface $uri URI
     * @param array $headers Optional headers
     * @param string|StreamInterface|null $body Optional body
     * @param string $protocolVersion Optional protocol version (default: "1.1")
     */
    public function __construct(
        string $method,
        $uri,
        array $headers = [],
        $body = null,
        string $protocolVersion = '1.1'
    ) {
        $this->method = strtoupper($method);
        $this->uri = is_string($uri) ? new Uri($uri) : $uri;
        $this->protocolVersion = $protocolVersion;

        // Normalize headers
        foreach ($headers as $name => $value) {
            $normalized = strtolower($name);
            $this->headers[$normalized] = is_array($value) ? $value : [$value];
        }

        // Set body
        if ($body !== null) {
            $this->body = is_string($body) ? new StringStream($body) : $body;
        } else {
            $this->body = null;
        }
    }

    // PSR-7 RequestInterface methods

    public function getRequestTarget(): string
    {
        if ($this->requestTarget !== '') {
            return $this->requestTarget;
        }

        $target = $this->uri->getPath();
        if ($target === '') {
            $target = '/';
        }

        if ($this->uri->getQuery() !== '') {
            $target .= '?' . $this->uri->getQuery();
        }

        return $target;
    }

    public function withRequestTarget($requestTarget): RequestInterface
    {
        $new = clone $this;
        $new->requestTarget = $requestTarget;
        return $new;
    }

    public function getMethod(): string
    {
        return $this->method;
    }

    public function withMethod($method): RequestInterface
    {
        $new = clone $this;
        $new->method = strtoupper($method);
        return $new;
    }

    public function getUri(): UriInterface
    {
        return $this->uri;
    }

    public function withUri(UriInterface $uri, $preserveHost = false): RequestInterface
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
        return $this->protocolVersion;
    }

    public function withProtocolVersion($version): RequestInterface
    {
        $new = clone $this;
        $new->protocolVersion = $version;
        return $new;
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

    public function withHeader($name, $value): RequestInterface
    {
        $normalized = strtolower($name);
        $new = clone $this;
        $new->headers[$normalized] = is_array($value) ? $value : [$value];
        return $new;
    }

    public function withAddedHeader($name, $value): RequestInterface
    {
        $normalized = strtolower($name);
        $new = clone $this;
        $new->headers[$normalized] = array_merge(
            $this->headers[$normalized] ?? [],
            is_array($value) ? $value : [$value]
        );
        return $new;
    }

    public function withoutHeader($name): RequestInterface
    {
        $normalized = strtolower($name);
        $new = clone $this;
        unset($new->headers[$normalized]);
        return $new;
    }

    public function getBody(): StreamInterface
    {
        return $this->body ?? new StringStream('');
    }

    public function withBody(StreamInterface $body): RequestInterface
    {
        $new = clone $this;
        $new->body = $body;
        return $new;
    }
}
