<?php

namespace Async\Network\Http;

use Psr\Http\Message\ResponseInterface;
use Psr\Http\Message\ServerRequestInterface;
use Psr\Http\Server\RequestHandlerInterface;

/**
 * Wraps a callable as a PSR-15 RequestHandler
 *
 * Allows using simple callables in the middleware chain:
 * function(ServerRequestInterface $request): ResponseInterface { ... }
 */
class CallableRequestHandler implements RequestHandlerInterface
{
    private $handler;

    /**
     * @param callable(ServerRequestInterface): ResponseInterface $handler
     */
    public function __construct(callable $handler)
    {
        $this->handler = $handler;
    }

    public function handle(ServerRequestInterface $request): ResponseInterface
    {
        return ($this->handler)($request);
    }
}
