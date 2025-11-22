<?php

namespace Async\Network\Http;

use Psr\Http\Message\ResponseInterface;
use Psr\Http\Message\ServerRequestInterface;
use Psr\Http\Server\MiddlewareInterface;
use Psr\Http\Server\RequestHandlerInterface;

/**
 * PSR-15 Middleware Stack
 *
 * Manages a chain of middleware that process requests in order.
 * Each middleware can:
 * - Process the request and call the next handler
 * - Short-circuit and return a response early
 * - Modify the request before passing to next handler
 * - Modify the response from next handler
 *
 * Example:
 * ```php
 * $stack = new MiddlewareStack();
 * $stack->add($loggingMiddleware);
 * $stack->add($authMiddleware);
 * $stack->add($corsMiddleware);
 *
 * $handler = $stack->build($finalHandler);
 * $response = $handler->handle($request);
 * ```
 */
class MiddlewareStack
{
    /**
     * @var MiddlewareInterface[]
     */
    private array $middlewares = [];

    /**
     * Add a middleware to the stack
     *
     * Middlewares are executed in the order they are added.
     *
     * @param MiddlewareInterface $middleware
     * @return self
     */
    public function add(MiddlewareInterface $middleware): self
    {
        $this->middlewares[] = $middleware;
        return $this;
    }

    /**
     * Build a request handler that wraps the final handler with all middlewares
     *
     * @param RequestHandlerInterface $finalHandler The final handler after all middleware
     * @return RequestHandlerInterface
     */
    public function build(RequestHandlerInterface $finalHandler): RequestHandlerInterface
    {
        // Build the handler chain in reverse order
        // Last middleware added will be closest to the final handler
        $handler = $finalHandler;

        for ($i = count($this->middlewares) - 1; $i >= 0; $i--) {
            $handler = new MiddlewareRequestHandler($this->middlewares[$i], $handler);
        }

        return $handler;
    }
}

/**
 * Internal handler that wraps a middleware with its next handler
 */
class MiddlewareRequestHandler implements RequestHandlerInterface
{
    private MiddlewareInterface $middleware;
    private RequestHandlerInterface $nextHandler;

    public function __construct(MiddlewareInterface $middleware, RequestHandlerInterface $nextHandler)
    {
        $this->middleware = $middleware;
        $this->nextHandler = $nextHandler;
    }

    public function handle(ServerRequestInterface $request): ResponseInterface
    {
        return $this->middleware->process($request, $this->nextHandler);
    }
}
