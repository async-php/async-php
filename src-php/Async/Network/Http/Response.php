<?php

namespace Async\Network\Http;

use Async\Kernel\Network\Http\HttpResponse as KernelResponse;
use Async\Kernel\IO\Reader;

class Response
{
    protected KernelResponse $kernel;

    public function __construct(int $status = 200, string|Reader|null $body = null)
    {
        $this->kernel = new KernelResponse();
        $this->kernel->set_status_code($status);

        if ($body !== null) {
            $this->kernel->set_body($body);
        }
    }

    /**
     * Set the HTTP status code
     */
    public function withStatus(int $status): self
    {
        $this->kernel->set_status_code($status);
        return $this;
    }

    /**
     * Get the HTTP status code
     */
    public function getStatus(): int
    {
        return $this->kernel->get_status_code();
    }

    /**
     * Set a header
     */
    public function withHeader(string $name, string $value): self
    {
        $this->kernel->set_header($name, $value);
        return $this;
    }

    /**
     * Get a header value
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
     * Set the response body
     *
     * @param string|Reader|null $body String content or Reader object for streaming
     */
    public function withBody(string|Reader|null $body): self
    {
        $this->kernel->set_body($body);
        return $this;
    }

    /**
     * Set body as JSON
     */
    public function withJson(mixed $data, int $options = 0): self
    {
        $json = json_encode($data, $options);
        if ($json === false) {
            throw new \RuntimeException('Failed to encode JSON: ' . json_last_error_msg());
        }

        $this->kernel->set_header('Content-Type', 'application/json');
        $this->kernel->set_body($json);
        return $this;
    }

    /**
     * Set body from a Reader object for streaming responses
     */
    public function withStreamBody(Reader $reader): self
    {
        $this->kernel->set_body($reader);
        return $this;
    }

    /**
     * Get the underlying kernel response
     */
    public function getKernel(): KernelResponse
    {
        return $this->kernel;
    }

    /**
     * Create a JSON response
     */
    public static function json(mixed $data, int $status = 200, int $options = 0): self
    {
        $response = new self($status);
        return $response->withJson($data, $options);
    }

    /**
     * Create a text response
     */
    public static function text(string $content, int $status = 200): self
    {
        $response = new self($status, $content);
        $response->withHeader('Content-Type', 'text/plain; charset=utf-8');
        return $response;
    }

    /**
     * Create an HTML response
     */
    public static function html(string $html, int $status = 200): self
    {
        $response = new self($status, $html);
        $response->withHeader('Content-Type', 'text/html; charset=utf-8');
        return $response;
    }

    /**
     * Create a redirect response
     */
    public static function redirect(string $url, int $status = 302): self
    {
        $response = new self($status);
        $response->withHeader('Location', $url);
        return $response;
    }
}
