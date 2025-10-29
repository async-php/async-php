<?php

namespace Async\Network\Http;

use Async\Kernel\Network\Http\HttpResponse as KernelResponse;
use Fiber;

class Response
{
    protected KernelResponse $kernel;

    public function __construct(int|KernelResponse $status = 200, Body|string|null $body = null)
    {
        if ($status instanceof KernelResponse) {
            $this->kernel = $status;
        } else {
            $this->kernel = new KernelResponse($status);
        }

        if ($body instanceof Body) {
            $this->kernel->set_body($body->getKernel());
        } elseif (is_string($body)) {
            $this->kernel->set_body_string($body);
        }
    }

    public function withStatus(int $status): self
    {
        $this->kernel->set_status_code($status);
        return $this;
    }

    public function withHeader(string $name, string $value): self
    {
        $this->kernel->set_header($name, $value);
        return $this;
    }

    public function withBody(Body|string|null $body): self
    {
        if ($body instanceof Body) {
            $this->kernel->set_body($body->getKernel());
        } elseif (is_string($body)) {
            $this->kernel->set_body_string($body);
        } else {
            $this->kernel->set_body(null);
        }
        return $this;
    }

    public function initStream(): void
    {
        $this->kernel->init_stream();
    }

    public function write(string $data): void
    {
        Fiber::suspend($this->kernel->write($data));
    }

    public function end(): void
    {
        Fiber::suspend($this->kernel->end());
    }

    public function getKernel(): KernelResponse
    {
        return $this->kernel;
    }
}
