<?php

namespace Async\Network\Http;

use Async\Kernel\Network\Http\HttpServer as KernelHttpServer;
use Async\Kernel\Network\Http\Http3Server as KernelHttp3Server;
use Async\Kernel\IO\AsyncReadWriter;
use Async\IO;
use Async\IO\TokioIO;
use Fiber;

/**
 * HttpServer - Unified HTTP server supporting HTTP/1.1, HTTP/2, and HTTP/3
 *
 * Usage:
 * ```php
 * $server = new Server();
 * $listener = Listener::bind('127.0.0.1:8080');
 *
 * while (true) {
 *     $conn = $listener->accept();
 *     go(function() use ($server, $conn) {
 *         $server->serve($conn, function(Request $req): Response {
 *             $resp = new Response();
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
    private KernelHttpServer $httpServer;
    private KernelHttp3Server $http3Server;

    public function __construct()
    {
        $this->httpServer = new KernelHttpServer();
        $this->http3Server = new KernelHttp3Server();
    }

    /**
     * Configure to only accept HTTP/1.1 connections
     */
    public function http1Only(): self
    {
        $this->httpServer->http1Only();
        return $this;
    }

    /**
     * Configure to only accept HTTP/2 connections
     */
    public function http2Only(): self
    {
        $this->httpServer->http2Only();
        return $this;
    }

    /**
     * Configure HTTP/1 settings
     *
     * @param array $options See KernelHttpServer::http1() for available options
     */
    public function http1(array $options): self
    {
        $this->httpServer->http1($options);
        return $this;
    }

    /**
     * Configure HTTP/2 settings
     *
     * @param array $options See KernelHttpServer::http2() for available options
     */
    public function http2(array $options): self
    {
        $this->httpServer->http2($options);
        return $this;
    }

    /**
     * Serve HTTP requests on a connection with zero-copy IO
     *
     * The handler receives a Request and must return a Response.
     * This uses the connection's native tokio IO for maximum performance.
     *
     * Supported connection types (zero-copy):
     * - Tcp\Socket::castTo(IO::READ|IO::WRITE) - HTTP/1.1 and HTTP/2
     * - Unix\Socket::castTo(IO::READ|IO::WRITE) - HTTP/1.1 and HTTP/2
     * - Tls\TlsStream::castTo(IO::READ|IO::WRITE) - HTTP/1.1 and HTTP/2
     * - FileSystem\FileHandle::castTo(IO::READ|IO::WRITE) - HTTP/1.1 and HTTP/2
     * - Quic\Connection - HTTP/3
     *
     * Also supports generic AsyncReadWriter from PHP bridges (with overhead)
     *
     * @param AsyncReadWriter|object $conn Connection IO
     * @param callable(Request): Response $handler Request handler
     * @return bool True if connection served successfully
     */
    public function serve($conn, callable $handler): bool
    {
        // Check for QUIC Connection (HTTP/3)
        if ($conn instanceof \Async\Network\Quic\Connection) {
            // Wrap the user handler to convert Request/Response to kernel types
            $kernelHandler = function($kernelRequest) use ($handler) {
                $request = Request::fromKernelRequest($kernelRequest);
                $response = $handler($request);
                return $response->toKernelResponse();
            };

            $future = $this->http3Server->serve($conn->unwrap(), $kernelHandler);
            $result = Fiber::suspend($future);
            return (bool)$result;
        }

        // Try to get AsyncReadWriter from connection
        if ($conn instanceof AsyncReadWriter) {
            $io = $conn;
        } elseif (is_callable([$conn, 'castTo'])) {
            // Use castTo to get AsyncReadWriter (works for TokioIO objects like Socket, TlsStream, etc.)
            $casted = $conn->castTo(IO::READ | IO::WRITE);

            // Check if result is TokioIO wrapper, unwrap it
            if ($casted instanceof TokioIO) {
                $io = $casted->unwrap();
            } elseif ($casted instanceof AsyncReadWriter) {
                $io = $casted;
            } else {
                throw new \InvalidArgumentException(
                    'castTo() must return TokioIO or AsyncReadWriter, got: ' . get_class($casted)
                );
            }
        } else {
            throw new \InvalidArgumentException(
                'Connection must be AsyncReadWriter, Quic\Connection, or support castTo(IO::READ|IO::WRITE)'
            );
        }

        if (!$io instanceof AsyncReadWriter) {
            throw new \InvalidArgumentException(
                'Failed to obtain AsyncReadWriter from connection, got: ' . get_class($io)
            );
        }

        // Wrap the user handler to convert Request/Response to kernel types
        $kernelHandler = function($kernelRequest) use ($handler) {
            $request = Request::fromKernelRequest($kernelRequest);
            $response = $handler($request);
            return $response->toKernelResponse();
        };

        // Serve the connection
        $future = $this->httpServer->serve($io, $kernelHandler);
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
     * @param callable(Request): Response $handler Request handler
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
     * Listen and serve on an address (static convenience method)
     *
     * This is a simple wrapper around listenAndServe() that can be used
     * without creating a Server instance first.
     *
     * @param string $addr Address to bind to (e.g., "127.0.0.1:8080")
     * @param callable(Request): Response $handler Request handler
     */
    public static function listen(string $addr, callable $handler): void
    {
        $server = new self();
        $server->listenAndServe($addr, $handler);
    }
}