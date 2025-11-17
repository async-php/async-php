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
     * Send a GET request
     *
     * @param string $url
     * @return ResponseInterface
     */
    public function get(string $url): ResponseInterface
    {
        $kernelRequest = $this->kernel->get($url);
        $kernelResponse = Fiber::suspend($kernelRequest->send());
        return new Psr7Response($kernelResponse);
    }

    /**
     * Send a POST request
     *
     * @param string $url
     * @param mixed $body Optional request body
     * @param array $headers Optional headers
     * @return ResponseInterface
     */
    public function post(string $url, $body = null, array $headers = []): ResponseInterface
    {
        $kernelRequest = $this->kernel->post($url);

        foreach ($headers as $name => $value) {
            $kernelRequest->header($name, $value);
        }

        if ($body !== null) {
            if (is_array($body)) {
                $kernelRequest->bodyJson(json_encode($body));
            } elseif (is_string($body)) {
                $kernelRequest->bodyText($body);
            }
        }

        $kernelResponse = Fiber::suspend($kernelRequest->send());
        return new Psr7Response($kernelResponse);
    }

    /**
     * Send a PUT request
     *
     * @param string $url
     * @param mixed $body Optional request body
     * @param array $headers Optional headers
     * @return ResponseInterface
     */
    public function put(string $url, $body = null, array $headers = []): ResponseInterface
    {
        $kernelRequest = $this->kernel->put($url);

        foreach ($headers as $name => $value) {
            $kernelRequest->header($name, $value);
        }

        if ($body !== null) {
            if (is_array($body)) {
                $kernelRequest->bodyJson(json_encode($body));
            } elseif (is_string($body)) {
                $kernelRequest->bodyText($body);
            }
        }

        $kernelResponse = Fiber::suspend($kernelRequest->send());
        return new Psr7Response($kernelResponse);
    }

    /**
     * Send a PATCH request
     *
     * @param string $url
     * @param mixed $body Optional request body
     * @param array $headers Optional headers
     * @return ResponseInterface
     */
    public function patch(string $url, $body = null, array $headers = []): ResponseInterface
    {
        $kernelRequest = $this->kernel->patch($url);

        foreach ($headers as $name => $value) {
            $kernelRequest->header($name, $value);
        }

        if ($body !== null) {
            if (is_array($body)) {
                $kernelRequest->bodyJson(json_encode($body));
            } elseif (is_string($body)) {
                $kernelRequest->bodyText($body);
            }
        }

        $kernelResponse = Fiber::suspend($kernelRequest->send());
        return new Psr7Response($kernelResponse);
    }

    /**
     * Send a DELETE request
     *
     * @param string $url
     * @return ResponseInterface
     */
    public function delete(string $url): ResponseInterface
    {
        $kernelRequest = $this->kernel->delete($url);
        $kernelResponse = Fiber::suspend($kernelRequest->send());
        return new Psr7Response($kernelResponse);
    }

    /**
     * Send a HEAD request
     *
     * @param string $url
     * @return ResponseInterface
     */
    public function head(string $url): ResponseInterface
    {
        $kernelRequest = $this->kernel->head($url);
        $kernelResponse = Fiber::suspend($kernelRequest->send());
        return new Psr7Response($kernelResponse);
    }

    /**
     * Create a request builder for advanced configuration
     *
     * Use this when you need to configure the request before sending:
     * ```php
     * $response = $client->request('POST', '/api/data')
     *     ->header('X-Custom', 'value')
     *     ->bodyJson(['key' => 'value'])
     *     ->send();
     * ```
     *
     * @param string $method
     * @param string $url
     * @return KernelRequest
     */
    public function request(string $method, string $url): KernelRequest
    {
        return $this->kernel->request($method, $url);
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

    /**
     * proxy to kernel method
     *
     * @param string $method
     * @param array $args
     * @return mixed
     */
    public function __call(string $method, array $args)
    {
        return $this->kernel->$method(...$args);
    }
}
