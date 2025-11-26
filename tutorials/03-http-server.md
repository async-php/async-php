# HTTP Server Tutorial

Build high-performance async HTTP servers with Async-PHP.

## Quick Start

```php
use Async\Kernel;
use Async\Network\Http\Server;
use Async\Network\Tcp\Listener;
use Async\Kernel\Network\Http\HttpRequest;
use Async\Kernel\Network\Http\HttpResponse;

Kernel::run(function () {
    $server = new Server();
    $listener = Listener::bind('127.0.0.1:8080');

    echo "Server listening on http://127.0.0.1:8080\n";

    while (true) {
        $conn = $listener->accept();

        go(function() use ($server, $conn) {
            $server->serve($conn, function(HttpRequest $req): HttpResponse {
                $resp = new HttpResponse();
                $resp->setStatus(200);
                $resp->setHeader('Content-Type', 'text/html');
                $resp->setBody('<h1>Hello, World!</h1>');
                return $resp;
            });
        });
    }
});
```

## Features

- HTTP/1.1 and HTTP/2 support
- Zero-copy I/O for high performance
- Concurrent request handling
- PSR-15 middleware support
- Request routing
- Streaming responses
- WebSocket support (via upgrade)

## Basic Server

### Hello World Server

```php
Kernel::run(function () {
    $server = new Server();
    $listener = Listener::bind('0.0.0.0:8080');

    while (true) {
        $conn = $listener->accept();

        go(function() use ($server, $conn) {
            try {
                $server->serve($conn, function(HttpRequest $req): HttpResponse {
                    $resp = new HttpResponse();
                    $resp->setStatus(200);
                    $resp->setHeader('Content-Type', 'text/plain');
                    $resp->setBody("Hello from Async PHP!\n");
                    return $resp;
                });
            } catch (Throwable $e) {
                echo "Error: {$e->getMessage()}\n";
            }
        });
    }
});
```

### Request Inspection

```php
$server->serve($conn, function(HttpRequest $req): HttpResponse {
    $method = $req->method();
    $path = $req->path();
    $query = $req->queryString();
    $headers = $req->getHeaders();
    $body = $req->body();

    echo "[$method] $path";
    if ($query) echo "?$query";
    echo "\n";

    $resp = new HttpResponse();
    $resp->setStatus(200);
    $resp->setHeader('Content-Type', 'application/json');
    $resp->setBody(json_encode([
        'method' => $method,
        'path' => $path,
        'query' => $query,
        'headers' => $headers
    ]));

    return $resp;
});
```

## Routing

### Simple Router

```php
function router(HttpRequest $req): HttpResponse
{
    $path = $req->path();
    $method = $req->method();

    $resp = new HttpResponse();

    return match ([$method, $path]) {
        ['GET', '/'] => homePage($resp),
        ['GET', '/about'] => aboutPage($resp),
        ['GET', '/api/users'] => getUsers($resp),
        ['POST', '/api/users'] => createUser($req, $resp),
        ['GET', preg_match('#^/api/users/(\d+)$#', $path, $m) ? $m : null]
            => getUser($m[1], $resp),
        default => notFound($resp)
    };
}

function homePage(HttpResponse $resp): HttpResponse
{
    $resp->setStatus(200);
    $resp->setHeader('Content-Type', 'text/html');
    $resp->setBody('<h1>Welcome Home</h1>');
    return $resp;
}

function notFound(HttpResponse $resp): HttpResponse
{
    $resp->setStatus(404);
    $resp->setHeader('Content-Type', 'application/json');
    $resp->setBody(json_encode(['error' => 'Not Found']));
    return $resp;
}
```

### Path Parameters

```php
function handleRequest(HttpRequest $req): HttpResponse
{
    $path = $req->path();

    if (preg_match('#^/users/(\d+)$#', $path, $matches)) {
        $userId = $matches[1];
        return getUserById($userId);
    }

    if (preg_match('#^/posts/([^/]+)/comments$#', $path, $matches)) {
        $postSlug = $matches[1];
        return getPostComments($postSlug);
    }

    return notFound();
}
```

### Query Parameters

```php
function handleApiRequest(HttpRequest $req): HttpResponse
{
    $query = $req->queryString();
    parse_str($query, $params);

    $page = $params['page'] ?? 1;
    $limit = $params['limit'] ?? 10;
    $sort = $params['sort'] ?? 'created_at';

    $data = fetchData($page, $limit, $sort);

    $resp = new HttpResponse();
    $resp->setStatus(200);
    $resp->setHeader('Content-Type', 'application/json');
    $resp->setBody(json_encode($data));
    return $resp;
}
```

## Request Body Handling

### JSON Requests

```php
function createUser(HttpRequest $req): HttpResponse
{
    $body = $req->body();
    $data = json_decode($body, true);

    if (!isset($data['name']) || !isset($data['email'])) {
        $resp = new HttpResponse();
        $resp->setStatus(400);
        $resp->setHeader('Content-Type', 'application/json');
        $resp->setBody(json_encode(['error' => 'Missing required fields']));
        return $resp;
    }

    // Create user in database
    $userId = createUserInDb($data['name'], $data['email']);

    $resp = new HttpResponse();
    $resp->setStatus(201);
    $resp->setHeader('Content-Type', 'application/json');
    $resp->setHeader('Location', "/api/users/$userId");
    $resp->setBody(json_encode(['id' => $userId]));
    return $resp;
}
```

### Form Data

```php
function handleFormSubmit(HttpRequest $req): HttpResponse
{
    $body = $req->body();
    parse_str($body, $formData);

    $username = $formData['username'] ?? '';
    $password = $formData['password'] ?? '';

    if (validateCredentials($username, $password)) {
        $resp = new HttpResponse();
        $resp->setStatus(302);
        $resp->setHeader('Location', '/dashboard');
        return $resp;
    }

    $resp = new HttpResponse();
    $resp->setStatus(401);
    $resp->setBody('Invalid credentials');
    return $resp;
}
```

## Response Types

### HTML Response

```php
$resp->setStatus(200);
$resp->setHeader('Content-Type', 'text/html; charset=utf-8');
$resp->setBody(<<<HTML
<!DOCTYPE html>
<html>
<head>
    <title>My Page</title>
</head>
<body>
    <h1>Welcome!</h1>
</body>
</html>
HTML);
```

### JSON Response

```php
$resp->setStatus(200);
$resp->setHeader('Content-Type', 'application/json');
$resp->setBody(json_encode([
    'status' => 'success',
    'data' => ['id' => 123]
]));
```

### File Downloads

```php
$fileContent = file_get_contents('/path/to/file.pdf');

$resp->setStatus(200);
$resp->setHeader('Content-Type', 'application/pdf');
$resp->setHeader('Content-Disposition', 'attachment; filename="document.pdf"');
$resp->setHeader('Content-Length', strlen($fileContent));
$resp->setBody($fileContent);
```

### Redirects

```php
$resp->setStatus(302);
$resp->setHeader('Location', '/new-path');
$resp->setBody('');
```

### Streaming Response

```php
// For large responses, set body in chunks
$resp->setStatus(200);
$resp->setHeader('Content-Type', 'text/plain');
$resp->setHeader('Transfer-Encoding', 'chunked');

$data = generateLargeDataset();
$resp->setBody($data);
```

## HTTP/2 Support

```php
$server = new Server();
$server->http2Only(); // Enable HTTP/2 only

$listener = Listener::bind('127.0.0.1:8080');

while (true) {
    $conn = $listener->accept();
    go(fn() => $server->serve($conn, $handler));
}
```

## HTTP/3 Support

HTTP/3 uses QUIC as the transport protocol, providing better performance and connection migration support.

### Prerequisites

HTTP/3 requires TLS certificates. Generate a self-signed certificate for testing:

```bash
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes
```

### Basic HTTP/3 Server

```php
use Async\Kernel\Network\Quic\QuicListener;
use Async\Kernel\Network\Http\Http3Server;

// Create QUIC listener (similar to TcpListener)
$listener = QuicListener::bind('127.0.0.1:4433', 'cert.pem', 'key.pem');

// Create HTTP/3 server
$server = new Http3Server();

// Accept loop (same pattern as HTTP/1.1 and HTTP/2)
while (true) {
    $conn = $listener->accept();

    go(function() use ($server, $conn) {
        $server->serve($conn, function(HttpRequest $req): HttpResponse {
            $resp = new HttpResponse();
            $resp->setStatus(200);
            $resp->setHeader('Content-Type', 'text/plain');
            $resp->setBody('Hello from HTTP/3!');
            return $resp;
        });
    });
}
```

### Testing HTTP/3

```bash
# Using curl with HTTP/3 support
curl --http3 https://localhost:4433/ --insecure

# Or use a modern browser (Chrome, Firefox with HTTP/3 enabled)
```

### HTTP/3 Features

- **QUIC Protocol**: UDP-based transport with built-in encryption
- **0-RTT**: Faster connection establishment for repeat connections
- **Connection Migration**: Connections survive network changes (e.g., WiFi to cellular)
- **Improved Performance**: Better congestion control and packet loss recovery
- **No Head-of-Line Blocking**: Independent streams don't block each other

## Middleware (PSR-15)

### Adding Middleware

```php
use Psr\Http\Server\MiddlewareInterface;
use Psr\Http\Server\RequestHandlerInterface;
use Psr\Http\Message\ServerRequestInterface;
use Psr\Http\Message\ResponseInterface;

class LoggingMiddleware implements MiddlewareInterface
{
    public function process(
        ServerRequestInterface $request,
        RequestHandlerInterface $handler
    ): ResponseInterface {
        $start = microtime(true);

        $response = $handler->handle($request);

        $duration = microtime(true) - $start;
        echo "Request took " . round($duration * 1000, 2) . "ms\n";

        return $response;
    }
}

// Add to server
$server = new Server();
$server->withMiddleware(new LoggingMiddleware());
```

### CORS Middleware

```php
class CorsMiddleware implements MiddlewareInterface
{
    public function process(
        ServerRequestInterface $request,
        RequestHandlerInterface $handler
    ): ResponseInterface {
        $response = $handler->handle($request);

        return $response
            ->withHeader('Access-Control-Allow-Origin', '*')
            ->withHeader('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE')
            ->withHeader('Access-Control-Allow-Headers', 'Content-Type');
    }
}
```

### Authentication Middleware

```php
class AuthMiddleware implements MiddlewareInterface
{
    public function process(
        ServerRequestInterface $request,
        RequestHandlerInterface $handler
    ): ResponseInterface {
        $token = $request->getHeaderLine('Authorization');

        if (!$this->validateToken($token)) {
            return new Response(401, [], json_encode([
                'error' => 'Unauthorized'
            ]));
        }

        return $handler->handle($request);
    }

    private function validateToken(string $token): bool
    {
        // Validate token logic
        return str_starts_with($token, 'Bearer ');
    }
}
```

## Real-World Example: REST API

```php
Kernel::run(function () {
    $server = new Server();
    $listener = Listener::bind('127.0.0.1:8080');

    // Simple in-memory store
    $users = [
        1 => ['id' => 1, 'name' => 'Alice', 'email' => 'alice@example.com'],
        2 => ['id' => 2, 'name' => 'Bob', 'email' => 'bob@example.com'],
    ];
    $nextId = 3;

    echo "REST API listening on http://127.0.0.1:8080\n";
    echo "Endpoints:\n";
    echo "  GET    /api/users\n";
    echo "  GET    /api/users/:id\n";
    echo "  POST   /api/users\n";
    echo "  PUT    /api/users/:id\n";
    echo "  DELETE /api/users/:id\n\n";

    while (true) {
        $conn = $listener->accept();

        go(function() use ($server, $conn, &$users, &$nextId) {
            $server->serve($conn, function(HttpRequest $req) use (&$users, &$nextId): HttpResponse {
                $method = $req->method();
                $path = $req->path();
                $resp = new HttpResponse();
                $resp->setHeader('Content-Type', 'application/json');

                // List users
                if ($method === 'GET' && $path === '/api/users') {
                    $resp->setStatus(200);
                    $resp->setBody(json_encode(array_values($users)));
                    return $resp;
                }

                // Get user by ID
                if ($method === 'GET' && preg_match('#^/api/users/(\d+)$#', $path, $m)) {
                    $id = (int)$m[1];
                    if (isset($users[$id])) {
                        $resp->setStatus(200);
                        $resp->setBody(json_encode($users[$id]));
                    } else {
                        $resp->setStatus(404);
                        $resp->setBody(json_encode(['error' => 'User not found']));
                    }
                    return $resp;
                }

                // Create user
                if ($method === 'POST' && $path === '/api/users') {
                    $data = json_decode($req->body(), true);
                    $user = [
                        'id' => $nextId++,
                        'name' => $data['name'] ?? '',
                        'email' => $data['email'] ?? ''
                    ];
                    $users[$user['id']] = $user;

                    $resp->setStatus(201);
                    $resp->setHeader('Location', "/api/users/{$user['id']}");
                    $resp->setBody(json_encode($user));
                    return $resp;
                }

                // Update user
                if ($method === 'PUT' && preg_match('#^/api/users/(\d+)$#', $path, $m)) {
                    $id = (int)$m[1];
                    if (isset($users[$id])) {
                        $data = json_decode($req->body(), true);
                        $users[$id] = array_merge($users[$id], $data);
                        $resp->setStatus(200);
                        $resp->setBody(json_encode($users[$id]));
                    } else {
                        $resp->setStatus(404);
                        $resp->setBody(json_encode(['error' => 'User not found']));
                    }
                    return $resp;
                }

                // Delete user
                if ($method === 'DELETE' && preg_match('#^/api/users/(\d+)$#', $path, $m)) {
                    $id = (int)$m[1];
                    if (isset($users[$id])) {
                        unset($users[$id]);
                        $resp->setStatus(204);
                        $resp->setBody('');
                    } else {
                        $resp->setStatus(404);
                        $resp->setBody(json_encode(['error' => 'User not found']));
                    }
                    return $resp;
                }

                // 404
                $resp->setStatus(404);
                $resp->setBody(json_encode(['error' => 'Not Found']));
                return $resp;
            });
        });
    }
});
```

## Best Practices

### 1. Use go() for Each Connection
```php
// ✅ Good - Non-blocking
while (true) {
    $conn = $listener->accept();
    go(fn() => $server->serve($conn, $handler));
}

// ❌ Bad - Blocks other connections
while (true) {
    $conn = $listener->accept();
    $server->serve($conn, $handler); // Blocks!
}
```

### 2. Handle Errors Properly
```php
go(function() use ($server, $conn) {
    try {
        $server->serve($conn, $handler);
    } catch (Throwable $e) {
        echo "Connection error: {$e->getMessage()}\n";
    }
});
```

### 3. Set Appropriate Headers
```php
$resp->setHeader('Content-Type', 'application/json; charset=utf-8');
$resp->setHeader('Cache-Control', 'no-cache');
$resp->setHeader('X-Content-Type-Options', 'nosniff');
```

### 4. Validate Input
```php
$data = json_decode($req->body(), true);
if (json_last_error() !== JSON_ERROR_NONE) {
    $resp->setStatus(400);
    $resp->setBody(json_encode(['error' => 'Invalid JSON']));
    return $resp;
}
```

## Performance Tips

- Use connection pooling for database connections
- Cache static responses
- Enable HTTP/2 for better performance
- Use zero-copy I/O when possible
- Keep handlers lightweight
- Offload heavy processing to background tasks

## Next Steps

- [Database Tutorial](04-pdo-database.md) - Integrate databases
- [Network Programming](05-network.md) - TCP/TLS/WebSockets
- [Async Patterns](06-async-patterns.md) - Advanced patterns
