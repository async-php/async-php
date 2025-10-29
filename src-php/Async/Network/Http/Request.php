<?php

namespace Async\Network\Http;

use Async\Kernel\Network\Http\HttpBody as KernelHttpBody;
use Async\Kernel\Network\Http\HttpRequest as KernelRequest;

class Request
{
    protected KernelRequest $kernel;

    public function __construct(KernelRequest $kernel)
    {
        $this->kernel = $kernel;
    }

    public function getMethod(): string
    {
        return $this->kernel->get_method();
    }

    public function getUri(): string
    {
        return $this->kernel->get_uri();
    }

    public function getHeader(string $name): ?string
    {
        return $this->kernel->get_header($name);
    }

    public function getBody(): ?Body
    {
        $stream = $this->kernel->get_body();
        if ($stream) {
            return new Body($stream);
        }
        return null;
    }

    public function setBody(Body|object|null $body): void
    {
        if ($body instanceof Body) {
            $this->kernel->set_body($body->getKernel());
            return;
        }

        if (is_object($body)) {
            $this->kernel->set_body(new KernelHttpBody($body));
            return;
        }

        $this->kernel->set_body(null);
    }

    public function getKernel(): KernelRequest
    {
        return $this->kernel;
    }
}
