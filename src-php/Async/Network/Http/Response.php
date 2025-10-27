<?php

namespace Async\Network\Http;

use Async\Kernel\Network\Http\Response as KernelResponse;
use Fiber;

class Response
{
    protected KernelResponse $kernel;

    public function __construct(int|KernelResponse $status = 200, string $body = '')
    {
        if ($status instanceof KernelResponse) {
            $this->kernel = $status;
        } else {
            $this->kernel = new KernelResponse();
            $this->kernel->withStatus($status);
            $this->kernel->withBody($body);
        }
    }

    public function withStatus(int $status): self
    {
        $this->kernel->withStatus($status);
        return $this;
    }

    public function withHeader(string $name, string $value): self
    {
        $this->kernel->withHeader($name, $value);
        return $this;
    }

    public function withBody(string $body): self
    {
        $this->kernel->withBody($body);
        return $this;
    }

    public function initStream(): void
    {
        $this->kernel->initStream();
    }

    public function write(string $data): void
    {
        Fiber::suspend($this->kernel->write($data));
    }

    public function end(): void
    {
        Fiber::suspend($this->kernel->end());
    }

    public function getRequest(): Request
    {
        $kernelRequest = $this->kernel->getRequest();
        return new Request($kernelRequest);
    }

    public function getKernel(): KernelResponse
    {
        return $this->kernel;
    }
}
