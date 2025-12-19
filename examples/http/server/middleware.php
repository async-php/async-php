<?php

require_once __DIR__ . '/../../../vendor/autoload.php';

use Async\Kernel;
use Async\Network\Http\Server;
use Async\Network\Http\CallableRequestHandler;
use Async\Network\Http\Psr7ServerRequest;
use Psr\Http\Message\ResponseInterface;
use Psr\Http\Message\ServerRequestInterface;
use Psr\Http\Server\MiddlewareInterface;
use Psr\Http\Server\RequestHandlerInterface;
use Async\Network\Http\Psr7Response;
use Async\Kernel\Network\Http\HttpResponse as KernelResponse;

/**
 * Example: Logging Middleware
 * Logs incoming requests and outgoing responses
 */
class LoggingMiddleware implements MiddlewareInterface
{
    public function process(ServerRequestInterface $request, RequestHandlerInterface $handler): ResponseInterface
    {
        $method = $request->getMethod();
        $uri = $request->getUri()->getPath();
        $start = microtime(true);

        echo "[LOG] --> {$method} {$uri}\n";

        $response = $handler->handle($request);

        $duration = round((microtime(true) - $start) * 1000, 2);
        $status = $response->getStatusCode();

        echo "[LOG] <-- {$status} {$method} {$uri} ({$duration}ms)\n";

        return $response;
    }
}

/**
 * Example: CORS Middleware
 * Adds CORS headers to responses
 */
class CorsMiddleware implements MiddlewareInterface
{
    public function process(ServerRequestInterface $request, RequestHandlerInterface $handler): ResponseInterface
    {
        // Handle preflight requests
        if ($request->getMethod() === 'OPTIONS') {
            $response = new Psr7Response(new KernelResponse());
            return $response->withStatus(204)
                ->withHeader('Access-Control-Allow-Origin', '*')
                ->withHeader('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS')
                ->withHeader('Access-Control-Allow-Headers', 'Content-Type, Authorization');
        }

        // Add CORS headers to normal responses
        $response = $handler->handle($request);

        return $response
            ->withHeader('Access-Control-Allow-Origin', '*')
            ->withHeader('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS')
            ->withHeader('Access-Control-Allow-Headers', 'Content-Type, Authorization');
    }
}

/**
 * Example: Auth Middleware
 * Simple token-based authentication
 */
class AuthMiddleware implements MiddlewareInterface
{
    private array $publicPaths;

    public function __construct(array $publicPaths = [])
    {
        $this->publicPaths = $publicPaths;
    }

    public function process(ServerRequestInterface $request, RequestHandlerInterface $handler): ResponseInterface
    {
        $path = $request->getUri()->getPath();

        // Skip auth for public paths
        if (in_array($path, $this->publicPaths)) {
            return $handler->handle($request);
        }

        // Check for Authorization header
        $authHeader = $request->getHeaderLine('Authorization');
        if (empty($authHeader) || !str_starts_with($authHeader, 'Bearer ')) {
            $response = new Psr7Response(new KernelResponse());
            return $response->withStatus(401)
                ->withHeader('Content-Type', 'application/json')
                ->withBody(new \Async\Network\Http\StringStream(
                    json_encode(['error' => 'Unauthorized'])
                ));
        }

        // In a real app, validate the token here
        $token = substr($authHeader, 7);
        if ($token !== 'secret-token') {
            $response = new Psr7Response(new KernelResponse());
            return $response->withStatus(403)
                ->withHeader('Content-Type', 'application/json')
                ->withBody(new \Async\Network\Http\StringStream(
                    json_encode(['error' => 'Invalid token'])
                ));
        }

        // Add user info to request attributes
        $request = $request->withAttribute('user', ['id' => 1, 'name' => 'John Doe']);

        return $handler->handle($request);
    }
}

/**
 * Example: Request Handler
 * Handles the actual business logic
 */
class ApiHandler implements RequestHandlerInterface
{
    public function handle(ServerRequestInterface $request): ResponseInterface
    {
        $method = $request->getMethod();
        $path = $request->getUri()->getPath();

        // Route handling
        if ($path === '/' && $method === 'GET') {
            return $this->handleHome($request);
        } elseif ($path === '/api/users' && $method === 'GET') {
            return $this->handleUsers($request);
        } elseif ($path === '/api/profile' && $method === 'GET') {
            return $this->handleProfile($request);
        } else {
            return $this->handleNotFound();
        }
    }

    private function handleHome(ServerRequestInterface $request): ResponseInterface
    {
        $response = new Psr7Response(new KernelResponse());
        return $response->withStatus(200)
            ->withHeader('Content-Type', 'text/html')
            ->withBody(new \Async\Network\Http\StringStream(
                '<h1>PSR-15 Middleware Demo</h1>' .
                '<p>Try these endpoints:</p>' .
                '<ul>' .
                '<li>GET / - This page (public)</li>' .
                '<li>GET /api/users - List users (requires auth)</li>' .
                '<li>GET /api/profile - User profile (requires auth)</li>' .
                '</ul>' .
                '<p>Use: <code>Authorization: Bearer secret-token</code></p>'
            ));
    }

    private function handleUsers(ServerRequestInterface $request): ResponseInterface
    {
        $users = [
            ['id' => 1, 'name' => 'Alice'],
            ['id' => 2, 'name' => 'Bob'],
            ['id' => 3, 'name' => 'Charlie'],
        ];

        $response = new Psr7Response(new KernelResponse());
        return $response->withStatus(200)
            ->withHeader('Content-Type', 'application/json')
            ->withBody(new \Async\Network\Http\StringStream(
                json_encode($users)
            ));
    }

    private function handleProfile(ServerRequestInterface $request): ResponseInterface
    {
        $user = $request->getAttribute('user');

        $response = new Psr7Response(new KernelResponse());
        return $response->withStatus(200)
            ->withHeader('Content-Type', 'application/json')
            ->withBody(new \Async\Network\Http\StringStream(
                json_encode([
                    'user' => $user,
                    'message' => 'This is your profile',
                ])
            ));
    }

    private function handleNotFound(): ResponseInterface
    {
        $response = new Psr7Response(new KernelResponse());
        return $response->withStatus(404)
            ->withHeader('Content-Type', 'application/json')
            ->withBody(new \Async\Network\Http\StringStream(
                json_encode(['error' => 'Not found'])
            ));
    }
}

// Run the server
Kernel::run(function () {
    echo "Starting PSR-15 HTTP Server on http://127.0.0.1:9002\n";
    echo "Try:\n";
    echo "  curl http://127.0.0.1:9002/\n";
    echo "  curl http://127.0.0.1:9002/api/users -H 'Authorization: Bearer secret-token'\n";
    echo "  curl http://127.0.0.1:9002/api/profile -H 'Authorization: Bearer secret-token'\n";
    echo "\n";

    // Create server with middleware
    $server = new Server();
    $server->withMiddleware(new LoggingMiddleware())
           ->withMiddleware(new CorsMiddleware())
           ->withMiddleware(new AuthMiddleware(['/'])); // '/' is public

    // Start listening
    $handler = new ApiHandler();
    $server->listenAndServePsr15('127.0.0.1:9002', $handler);
});
