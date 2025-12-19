<?php

namespace Async\Network\Http;

use Async\Kernel\Network\Http\HttpClient as KernelClient;
use Async\Kernel\Network\Http\HttpRequest as KernelRequest;
use Async\Kernel\Network\Http\HttpResponse as KernelResponse;
use Fiber;

/**
 * HTTP Client - Async HTTP/1.1 and HTTP/2 client
 *
 * This client wraps the reqwest-based kernel client and provides
 * a convenient PHP API with fluent method chaining.
 *
 * Usage:
 * ```php
 * $client = new Client(['timeout' => 30]);
 * $response = $client->get('https://example.com');
 * echo $response->text();
 * ```
 */
class Client
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
     * @param array $headers Optional headers
     * @return KernelResponse
     */
    public function get(string $url, array $headers = []): KernelResponse
    {
        $kernelRequest = new KernelRequest('GET', $url);

        foreach ($headers as $name => $value) {
            $kernelRequest->header($name, $value);
        }

        return Fiber::suspend($this->kernel->send($kernelRequest));
    }

    /**
     * Send a POST request
     *
     * @param string $url
     * @param mixed $body Optional request body (string or array for JSON)
     * @param array $headers Optional headers
     * @return KernelResponse
     */
    public function post(string $url, $body = null, array $headers = []): KernelResponse
    {
        $kernelRequest = new KernelRequest('POST', $url);

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

        return Fiber::suspend($this->kernel->send($kernelRequest));
    }

    /**
     * Send a PUT request
     *
     * @param string $url
     * @param mixed $body Optional request body (string or array for JSON)
     * @param array $headers Optional headers
     * @return KernelResponse
     */
    public function put(string $url, $body = null, array $headers = []): KernelResponse
    {
        $kernelRequest = new KernelRequest('PUT', $url);

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

        return Fiber::suspend($this->kernel->send($kernelRequest));
    }

    /**
     * Send a PATCH request
     *
     * @param string $url
     * @param mixed $body Optional request body (string or array for JSON)
     * @param array $headers Optional headers
     * @return KernelResponse
     */
    public function patch(string $url, $body = null, array $headers = []): KernelResponse
    {
        $kernelRequest = new KernelRequest('PATCH', $url);

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

        return Fiber::suspend($this->kernel->send($kernelRequest));
    }

    /**
     * Send a DELETE request
     *
     * @param string $url
     * @param array $headers Optional headers
     * @return KernelResponse
     */
    public function delete(string $url, array $headers = []): KernelResponse
    {
        $kernelRequest = new KernelRequest('DELETE', $url);

        foreach ($headers as $name => $value) {
            $kernelRequest->header($name, $value);
        }

        return Fiber::suspend($this->kernel->send($kernelRequest));
    }

    /**
     * Send a HEAD request
     *
     * @param string $url
     * @param array $headers Optional headers
     * @return KernelResponse
     */
    public function head(string $url, array $headers = []): KernelResponse
    {
        $kernelRequest = new KernelRequest('HEAD', $url);

        foreach ($headers as $name => $value) {
            $kernelRequest->header($name, $value);
        }

        return Fiber::suspend($this->kernel->send($kernelRequest));
    }

    /**
     * Create a request builder for advanced configuration
     *
     * Use this when you need to configure the request before sending:
     * ```php
     * $request = $client->request('POST', '/api/data')
     *     ->header('X-Custom', 'value')
     *     ->bodyJson(['key' => 'value']);
     * $response = $client->send($request);
     * ```
     *
     * @param string $method
     * @param string $url
     * @return KernelRequest
     */
    public function request(string $method, string $url): KernelRequest
    {
        return new KernelRequest($method, $url);
    }

    /**
     * Send a KernelRequest
     *
     * @param KernelRequest $request
     * @return KernelResponse
     */
    public function send(KernelRequest $request): KernelResponse
    {
        return Fiber::suspend($this->kernel->send($request));
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
