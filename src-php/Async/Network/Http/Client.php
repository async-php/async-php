<?php

namespace Async\Network\Http;

use Async\Kernel\Network\Http\HttpClient as KernelClient;
use Async\Kernel\Network\Http\HttpRequest as KernelRequest;
use Async\Kernel\Network\Http\HttpResponse as KernelResponse;
use Fiber;

class Client
{
    private KernelClient $kernel;
    private array $defaultHeaders = [];

    public function __construct()
    {
        $this->kernel = new KernelClient();

        // Set default headers
        $this->defaultHeaders = [
            'User-Agent' => 'async-php/1.0',
            'Accept' => '*/*',
        ];
    }

    /**
     * Set the request timeout in seconds
     */
    public function setTimeout(int $seconds): self
    {
        $this->kernel->setTimeout($seconds);
        return $this;
    }

    /**
     * Get the timeout in seconds
     */
    public function getTimeout(): ?int
    {
        return $this->kernel->getTimeout();
    }

    /**
     * Enable or disable following redirects
     */
    public function setFollowRedirects(bool $follow): self
    {
        $this->kernel->setFollowRedirects($follow);
        return $this;
    }

    /**
     * Check if redirects are followed
     */
    public function getFollowRedirects(): bool
    {
        return $this->kernel->getFollowRedirects();
    }

    /**
     * Set maximum number of redirects to follow
     */
    public function setMaxRedirects(int $max): self
    {
        $this->kernel->setMaxRedirects($max);
        return $this;
    }

    /**
     * Get maximum number of redirects
     */
    public function getMaxRedirects(): int
    {
        return $this->kernel->getMaxRedirects();
    }

    /**
     * Set a default header for all requests
     */
    public function setDefaultHeader(string $name, string $value): self
    {
        $this->defaultHeaders[$name] = $value;
        return $this;
    }

    /**
     * Remove a default header
     */
    public function removeDefaultHeader(string $name): self
    {
        unset($this->defaultHeaders[$name]);
        return $this;
    }

    /**
     * Send a request and return the response
     *
     * @param string $method HTTP method (GET, POST, PUT, DELETE, etc.)
     * @param string $url Request URL
     * @param array $options Request options:
     *   - headers: array of headers
     *   - body: request body (string or null)
     *   - query: array of query parameters to append to URL
     *   - json: data to send as JSON (sets Content-Type and encodes body)
     * @return ClientResponse
     */
    public function request(string $method, string $url, array $options = []): ClientResponse
    {
        // Build URL with query parameters
        if (isset($options['query']) && is_array($options['query'])) {
            $queryString = http_build_query($options['query']);
            if ($queryString !== '') {
                $url .= (strpos($url, '?') === false ? '?' : '&') . $queryString;
            }
        }

        // Create kernel request
        $kernelRequest = new KernelRequest(strtoupper($method), $url);

        // Apply default headers
        foreach ($this->defaultHeaders as $name => $value) {
            $kernelRequest->setHeader($name, $value);
        }

        // Apply custom headers
        if (isset($options['headers']) && is_array($options['headers'])) {
            foreach ($options['headers'] as $name => $value) {
                $kernelRequest->setHeader($name, (string)$value);
            }
        }

        // Handle body
        $body = null;
        if (isset($options['json'])) {
            // JSON body
            $json = json_encode($options['json']);
            if ($json === false) {
                throw new \RuntimeException('Failed to encode JSON: ' . json_last_error_msg());
            }
            $body = $json;
            $kernelRequest->setHeader('Content-Type', 'application/json');
        } elseif (isset($options['body'])) {
            $body = $options['body'];
        }

        if ($body !== null) {
            // Wrap string body in StringReader to implement ReadCloser interface
            if (is_string($body)) {
                $body = StringReader::fromString($body);
            }
            $kernelRequest->setBody($body);
        }

        // Send request
        $future = $this->kernel->send($kernelRequest);
        $kernelResponse = Fiber::suspend($future);

        return new ClientResponse($kernelResponse);
    }

    /**
     * Send a GET request
     */
    public function get(string $url, array $options = []): ClientResponse
    {
        return $this->request('GET', $url, $options);
    }

    /**
     * Send a POST request
     */
    public function post(string $url, array $options = []): ClientResponse
    {
        return $this->request('POST', $url, $options);
    }

    /**
     * Send a PUT request
     */
    public function put(string $url, array $options = []): ClientResponse
    {
        return $this->request('PUT', $url, $options);
    }

    /**
     * Send a PATCH request
     */
    public function patch(string $url, array $options = []): ClientResponse
    {
        return $this->request('PATCH', $url, $options);
    }

    /**
     * Send a DELETE request
     */
    public function delete(string $url, array $options = []): ClientResponse
    {
        return $this->request('DELETE', $url, $options);
    }

    /**
     * Send a HEAD request
     */
    public function head(string $url, array $options = []): ClientResponse
    {
        return $this->request('HEAD', $url, $options);
    }

    /**
     * Send a OPTIONS request
     */
    public function options(string $url, array $options = []): ClientResponse
    {
        return $this->request('OPTIONS', $url, $options);
    }

    /**
     * Static helper to quickly create and send a GET request
     */
    public static function quickGet(string $url, array $options = []): ClientResponse
    {
        $client = new self();
        return $client->get($url, $options);
    }

    /**
     * Static helper to quickly create and send a POST request
     */
    public static function quickPost(string $url, array $options = []): ClientResponse
    {
        $client = new self();
        return $client->post($url, $options);
    }
}
