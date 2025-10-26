<?php

namespace Async\Network\Http;

use Async\Kernel\Network\Http\Request as KernelRequest;

class Request
{
    protected KernelRequest $kernel;

    public function __construct(KernelRequest $kernel)
    {
        $this->kernel = $kernel;
    }

    public function getMethod(): string
    {
        return $this->kernel->getMethod();
    }

    public function getUri(): string
    {
        return $this->kernel->getUri();
    }

    public function getHeader(string $name): ?string
    {
        return $this->kernel->getHeader($name);
    }

    public function getBodyStream(): ?RequestBody
    {
        $stream = $this->kernel->getBody();
        if ($stream) {
            return new RequestBody($stream);
        }
        return null;
    }

    public function getResponse(): Response
    {
        $kernelResponse = $this->kernel->getResponse();
        return new Response($kernelResponse);
    }
}
