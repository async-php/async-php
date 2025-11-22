<?php

namespace Async\Network\Http;

use Async\Kernel\Network\Http\HttpServer as KernelHttpServer;
use Async\Kernel\Network\Http\HttpRequest as KernelHttpRequest;
use Async\Kernel\Network\Http\HttpResponse as KernelHttpResponse;
use Async\Kernel\IO\AsyncReadWriter;
use Fiber;
use Psr\Http\Server\MiddlewareInterface;
use Psr\Http\Server\RequestHandlerInterface;

/**
 * HttpServer - HTTP/1.1 and HTTP/2 server with zero-copy IO
 *
 * Usage (Go-style):
 * ```php
 * $server = new HttpServer();
 * $listener = TcpListener::bind('127.0.0.1:8080');
 *
 * while (true) {
 *     $conn = $listener->accept();
 *     go(function() use ($server, $conn) {
 *         $server->serve($conn, function($req) {
 *             $resp = new HttpResponse();
 *             $resp->setStatus(200);
 *             $resp->setBody("Hello, World!");
 *             return $resp;
 *         });
 *     });
 * }
 * ```
 */
class Server
{
    private KernelHttpServer $builder;
    private ?MiddlewareStack $middlewareStack = null;

    public function __construct()
    {
        $this->builder = new KernelHttpServer();
    }

    /**
     * Configure to only accept HTTP/1.1 connections
     */
    public function http1Only(): self
    {
        $this->builder->http1Only();
        return $this;
    }

    /**
     * Configure to only accept HTTP/2 connections
     */
    public function http2Only(): self
    {
        $this->builder->http2Only();
        return $this;
    }

    /**
     * Configure HTTP/1 settings
     *
     * @param array $options See KernelHttpServer::http1() for available options
     */
    public function http1(array $options): self
    {
        $this->builder->http1($options);
        return $this;
    }

    /**
     * Configure HTTP/2 settings
     *
     * @param array $options See KernelHttpServer::http2() for available options
     */
    public function http2(array $options): self
    {
        $this->builder->http2($options);
        return $this;
    }

    /**
     * Add PSR-15 middleware to the server
     *
     * Middlewares are executed in the order they are added.
     * Use this to add cross-cutting concerns like logging, auth, CORS, etc.
     *
     * @param MiddlewareInterface $middleware
     * @return self
     */
    public function withMiddleware(MiddlewareInterface $middleware): self
    {
        if ($this->middlewareStack === null) {
            $this->middlewareStack = new MiddlewareStack();
        }
        $this->middlewareStack->add($middleware);
        return $this;
    }

    /**
     * Serve HTTP requests on a connection with zero-copy IO
     *
     * The handler receives an HttpRequest and must return an HttpResponse.
     * This uses the connection's native tokio IO for maximum performance.
     *
     * Supported connection types (zero-copy):
     * - Tcp\Socket::asReadWriter()
     * - Unix\Socket::asReadWriter()
     * - Tls\TlsStream::asReadWriter()
     * - FileSystem\FileHandle::asReadWriter()
     *
     * Also supports generic AsyncReadWriter from PHP bridges (with overhead)
     *
     * @param AsyncReadWriter $conn Connection IO
     * @param callable(KernelHttpRequest): KernelHttpResponse $handler Request handler
     * @return bool True if connection served successfully
     */
    public function serve($conn, callable $handler): bool
    {
        // If $conn is a socket/stream object with asReadWriter(), use it
        if (method_exists($conn, 'asReadWriter')) {
            $io = $conn->asReadWriter();
        } elseif ($conn instanceof AsyncReadWriter) {
            $io = $conn;
        } else {
            throw new \InvalidArgumentException(
                'Connection must be AsyncReadWriter or have asReadWriter() method'
            );
        }

        // Serve the connection
        $future = $this->builder->serve($io, $handler);
        $result = Fiber::suspend($future);

        return (bool)$result;
    }

    /**
     * Serve HTTP requests using PSR-15 middleware and handler
     *
     * This method wraps PSR-7 ServerRequestInterface and ResponseInterface,
     * allowing you to use standard PSR-15 middleware and handlers.
     *
     * @param AsyncReadWriter $conn Connection IO
     * @param RequestHandlerInterface $handler PSR-15 request handler
     * @return bool True if connection served successfully
     */
    public function servePsr15($conn, RequestHandlerInterface $handler): bool
    {
        // Build the handler chain with middleware (if any)
        $finalHandler = $handler;
        if ($this->middlewareStack !== null) {
            $finalHandler = $this->middlewareStack->build($handler);
        }

        // Wrap the PSR-15 handler to work with kernel request/response
        $kernelHandler = function(KernelHttpRequest $kernelRequest) use ($finalHandler): KernelHttpResponse {
            // Convert kernel request to PSR-7 server request
            $psr7Request = new Psr7ServerRequest($kernelRequest);

            // Process through PSR-15 handler chain
            $psr7Response = $finalHandler->handle($psr7Request);

            // Convert PSR-7 response back to kernel response
            return ResponseConverter::toKernelResponse($psr7Response);
        };

        // Use the standard serve method with our wrapper
        return $this->serve($conn, $kernelHandler);
    }

    /**
     * Listen and serve on an address (convenience method)
     *
     * This is equivalent to:
     * ```php
     * $listener = TcpListener::bind($addr);
     * while (true) {
     *     $conn = $listener->accept();
     *     go(function() use ($server, $conn, $handler) {
     *         $server->serve($conn, $handler);
     *     });
     * }
     * ```
     *
     * @param string $addr Address to bind to (e.g., "127.0.0.1:8080")
     * @param callable(KernelHttpRequest): KernelHttpResponse $handler Request handler
     */
    public function listenAndServe(string $addr, callable $handler): void
    {
        $listener = \Async\Network\Tcp\Listener::bind($addr);

        while (true) {
            $conn = $listener->accept();

            // Spawn a new fiber to handle this connection
            go(function() use ($conn, $handler) {
                try {
                    $this->serve($conn, $handler);
                } catch (\Throwable $e) {
                    // Log error but don't crash the server
                    error_log("HTTP server error: " . $e->getMessage());
                }
            });
        }
    }

    /**
     * Listen and serve using PSR-15 handler (convenience method)
     *
     * @param string $addr Address to bind to (e.g., "127.0.0.1:8080")
     * @param RequestHandlerInterface $handler PSR-15 request handler
     */
    public function listenAndServePsr15(string $addr, RequestHandlerInterface $handler): void
    {
        $listener = \Async\Network\Tcp\Listener::bind($addr);

        while (true) {
            $conn = $listener->accept();

            // Spawn a new fiber to handle this connection
            go(function() use ($conn, $handler) {
                try {
                    $this->servePsr15($conn, $handler);
                } catch (\Throwable $e) {
                    // Log error but don't crash the server
                    error_log("HTTP server error: " . $e->getMessage());
                }
            });
        }
    }
}
