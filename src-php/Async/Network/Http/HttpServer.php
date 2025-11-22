<?php

namespace Async\Network\Http;

use Async\Kernel\Network\Http\ConnectionBuilder;
use Async\Kernel\Network\Http\HttpRequest as KernelHttpRequest;
use Async\Kernel\Network\Http\HttpResponse as KernelHttpResponse;
use Async\Kernel\IO\AsyncReadWriter;
use Fiber;

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
class HttpServer
{
    private ConnectionBuilder $builder;

    public function __construct()
    {
        $this->builder = new ConnectionBuilder();
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
     * @param array $options See ConnectionBuilder::http1() for available options
     */
    public function http1(array $options): self
    {
        $this->builder->http1($options);
        return $this;
    }

    /**
     * Configure HTTP/2 settings
     *
     * @param array $options See ConnectionBuilder::http2() for available options
     */
    public function http2(array $options): self
    {
        $this->builder->http2($options);
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
}
