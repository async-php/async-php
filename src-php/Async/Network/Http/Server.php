<?php

namespace Async\Network\Http;

use Async\Kernel\Network\Http\HttpServer as KernelServer;
use Async\Kernel\Network\Http\HttpRequest as KernelRequest;
use Async\Kernel\Network\Http\HttpResponse as KernelResponse;
use Async\Kernel\Network\Http\HttpConnection;
use Async\Network\Tcp\Listener;

class Server
{
    private KernelServer $kernel;
    private ?string $certPem = null;
    private ?string $keyPem = null;

    public function __construct()
    {
        $this->kernel = new KernelServer();
    }

    /**
     * Set TLS certificate and key (PEM format strings)
     */
    public function setTls(string $certPem, string $keyPem): self
    {
        $this->certPem = $certPem;
        $this->keyPem = $keyPem;
        $this->kernel->setTls($certPem, $keyPem);
        return $this;
    }

    /**
     * Load TLS from files
     */
    public function setTlsFromFiles(string $certPath, string $keyPath): self
    {
        $certPem = file_get_contents($certPath);
        $keyPem = file_get_contents($keyPath);

        if ($certPem === false || $keyPem === false) {
            throw new \RuntimeException("Failed to read certificate or key file");
        }

        return $this->setTls($certPem, $keyPem);
    }

    /**
     * Enable or disable HTTP/1.1
     */
    public function setEnableHttp1(bool $enable): self
    {
        $this->kernel->setEnableHttp1($enable);
        return $this;
    }

    /**
     * Enable or disable HTTP/2
     */
    public function setEnableHttp2(bool $enable): self
    {
        $this->kernel->setEnableHttp2($enable);
        return $this;
    }

    /**
     * Enable or disable HTTP/3
     */
    public function setEnableHttp3(bool $enable): self
    {
        $this->kernel->setEnableHttp3($enable);
        return $this;
    }

    /**
     * Start listening on the given address
     *
     * @param string $addr Address to bind (e.g., "127.0.0.1:8080")
     * @param callable $handler Request handler function(Request): Response
     */
    public function listen(string $addr, callable $handler): void
    {
        // Create a handler object that Rust can call
        $handlerObject = new class($handler) {
            private $callback;

            public function __construct(callable $callback)
            {
                $this->callback = $callback;
            }

            public function handle(KernelRequest $kernelRequest): KernelResponse
            {
                // Wrap kernel request in user-friendly Request object
                $request = new Request($kernelRequest);

                // Call user handler
                $response = ($this->callback)($request);

                // If handler returns Response object, extract kernel response
                if ($response instanceof Response) {
                    return $response->getKernel();
                }

                // If handler returns KernelResponse directly, use it
                if ($response instanceof KernelResponse) {
                    return $response;
                }

                // Otherwise create a simple response with the returned value
                $kernelResponse = new KernelResponse();
                $kernelResponse->set_status_code(200);

                if (is_string($response)) {
                    $kernelResponse->set_body($response);
                } elseif ($response !== null) {
                    $kernelResponse->set_body((string)$response);
                }

                return $kernelResponse;
            }
        };

        $future = $this->kernel->listen($addr, $handlerObject);
        \Fiber::suspend($future);
    }

    /**
     * Static helper to quickly start a server
     */
    public static function create(string $addr, callable $handler, array $options = []): void
    {
        $server = new self();

        // Apply options
        if (isset($options['tls'])) {
            if (isset($options['tls']['cert']) && isset($options['tls']['key'])) {
                $server->setTls($options['tls']['cert'], $options['tls']['key']);
            } elseif (isset($options['tls']['cert_file']) && isset($options['tls']['key_file'])) {
                $server->setTlsFromFiles($options['tls']['cert_file'], $options['tls']['key_file']);
            }
        }

        if (isset($options['http1'])) {
            $server->setEnableHttp1($options['http1']);
        }

        if (isset($options['http2'])) {
            $server->setEnableHttp2($options['http2']);
        }

        if (isset($options['http3'])) {
            $server->setEnableHttp3($options['http3']);
        }

        $server->listen($addr, $handler);
    }

    /**
     * Start a simple HTTP/1.1 server using the new Go-style architecture
     *
     * This method provides a low-level, connection-oriented server that:
     * - Gives you full control over connection handling
     * - Only supports HTTP/1.1 (no HTTP/2 or HTTP/3)
     * - Uses the new HttpConnection API
     * - Follows Go's net/http pattern
     *
     * Example:
     * ```php
     * Server::serve('0.0.0.0:8080', function($request) {
     *     return Response::text('Hello World');
     * });
     * ```
     *
     * @param string $addr Address to bind (e.g., "0.0.0.0:8080")
     * @param callable $handler Request handler function(Request): Response
     */
    public static function serve(string $addr, callable $handler): never
    {
        $listener = Listener::bind($addr);
        echo "HTTP server listening on $addr\n";

        while (true) {
            $conn = $listener->accept();

            // Spawn a coroutine to handle this connection
            go(function () use ($conn, $handler) {
                $httpConn = HttpConnection::from($conn);

                // Handle multiple requests on the same connection (HTTP keep-alive)
                while (true) {
                    $future = $httpConn->readRequest();
                    $kernelRequest = \Fiber::suspend($future);

                    // Connection closed
                    if ($kernelRequest === null) {
                        break;
                    }

                    // Wrap kernel request in user-friendly Request object
                    $request = new Request($kernelRequest);

                    // Call user handler
                    $response = $handler($request);

                    // Convert Response to KernelResponse
                    $kernelResponse = null;
                    if ($response instanceof Response) {
                        $kernelResponse = $response->getKernel();
                    } elseif ($response instanceof KernelResponse) {
                        $kernelResponse = $response;
                    } else {
                        // Create a simple 200 OK response
                        $kernelResponse = new KernelResponse(200);
                        if (is_string($response)) {
                            $kernelResponse->setBody($response);
                        } elseif ($response !== null) {
                            $kernelResponse->setBody((string)$response);
                        }
                    }

                    // Write response
                    $future = $httpConn->writeResponse($kernelResponse);
                    \Fiber::suspend($future);
                }

                // Close the connection
                $conn->close();
            });
        }
    }
}
