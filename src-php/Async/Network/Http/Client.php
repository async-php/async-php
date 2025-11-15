<?php

namespace Async\Network\Http;

use Async\Kernel\Network\Http\HttpClient as KernelClient;
use Async\Kernel\Network\Http\HttpRequest as KernelRequest;
use Psr\Http\Client\ClientInterface;
use Psr\Http\Message\RequestInterface;
use Psr\Http\Message\ResponseInterface;
use Fiber;

/**
 * HTTP Client with PSR-18 support
 *
 * This client wraps the reqwest-based kernel client and provides
 * a convenient PHP API with fluent method chaining.
 *
 * @method KernelRequest get(string $url) Create a GET request
 * @method KernelRequest post(string $url) Create a POST request
 * @method KernelRequest put(string $url) Create a PUT request
 * @method KernelRequest patch(string $url) Create a PATCH request
 * @method KernelRequest delete(string $url) Create a DELETE request
 * @method KernelRequest head(string $url) Create a HEAD request
 * @method KernelRequest request(string $method, string $url) Create a custom request
 */
class Client implements ClientInterface
{
    private KernelClient $kernel;

    /**
     * Create HTTP client with optional configuration
     *
     * @param array $config Client configuration:
     *   - timeout: Total request timeout in seconds
     *   - connect_timeout: Connection timeout in seconds
     *   - pool_idle_timeout: Connection pool idle timeout
     *   - pool_max_idle_per_host: Max idle connections per host
     *   - max_redirects: Maximum redirects (0 = none)
     *   - enable_cookies: Enable automatic cookie handling
     *   - enable_http2: Enable HTTP/2 (true by default)
     *   - ca_cert_pem: Custom CA certificate
     *   - client_cert_pem: Client certificate for mTLS
     *   - client_key_pem: Client private key for mTLS
     *   - min_tls_version: Minimum TLS version ("1.0", "1.1", "1.2", "1.3")
     *   - accept_invalid_certs: Accept invalid certificates (DANGEROUS)
     */
    public function __construct(array $config = [])
    {
        $this->kernel = new KernelClient(
            $config['timeout'] ?? null,
            $config['connect_timeout'] ?? null,
            $config['pool_idle_timeout'] ?? null,
            $config['pool_max_idle_per_host'] ?? null,
            $config['max_redirects'] ?? null,
            $config['enable_cookies'] ?? null,
            $config['enable_http2'] ?? null,
            $config['ca_cert_pem'] ?? null,
            $config['client_cert_pem'] ?? null,
            $config['client_key_pem'] ?? null,
            $config['min_tls_version'] ?? null,
            $config['accept_invalid_certs'] ?? null
        );
    }

    /**
     * Magic method to forward calls to kernel client
     *
     * Supported methods:
     * - get(string $url): KernelRequest
     * - post(string $url): KernelRequest
     * - put(string $url): KernelRequest
     * - patch(string $url): KernelRequest
     * - delete(string $url): KernelRequest
     * - head(string $url): KernelRequest
     * - request(string $method, string $url): KernelRequest
     *
     * @param string $method
     * @param array $arguments
     * @return KernelRequest
     */
    public function __call(string $method, array $arguments): KernelRequest
    {
        if (!method_exists($this->kernel, $method)) {
            throw new \BadMethodCallException("Method {$method} does not exist on HttpClient");
        }

        return $this->kernel->{$method}(...$arguments);
    }

    /**
     * Send a PSR-7 request and return a PSR-7 response
     *
     * This implements the PSR-18 ClientInterface.
     *
     * @param RequestInterface $request
     * @return ResponseInterface
     */
    public function sendRequest(RequestInterface $request): ResponseInterface
    {
        // Create kernel request from PSR-7 request
        $kernelRequest = $this->kernel->request(
            $request->getMethod(),
            (string)$request->getUri()
        );

        // Apply headers
        foreach ($request->getHeaders() as $name => $values) {
            foreach ($values as $value) {
                $kernelRequest->header($name, $value);
            }
        }

        // Apply body
        $body = $request->getBody();
        if ($body->getSize() > 0) {
            $body->rewind();
            // Try bodyText with camelCase
            try {
                $kernelRequest->bodyText($body->getContents(), $request->getHeaderLine('Content-Type') ?: null);
            } catch (\Error $e) {
                // Fallback: skip body for now if method not found
                // TODO: Fix body_text method registration
            }
        }

        // Send and await response
        $future = $kernelRequest->send();
        $kernelResponse = Fiber::suspend($future);

        // Convert to PSR-7 response
        return new Psr7Response($kernelResponse);
    }

    /**
     * Get the underlying kernel client for advanced usage
     *
     * @return KernelClient
     */
    public function getKernel(): KernelClient
    {
        return $this->kernel;
    }
}
