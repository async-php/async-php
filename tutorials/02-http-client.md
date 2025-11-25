# HTTP Client Tutorial

Learn how to make async HTTP requests with Async-PHP's powerful HTTP client.

## Quick Start

```php
use Async\Kernel;
use Async\Network\Http\Client;

Kernel::run(function () {
    $client = Client::new();
    $response = $client->get('https://api.github.com/users/github');

    echo "Status: {$response->status()}\n";
    echo "Body: {$response->text()}\n";
});
```

## Features

- HTTP/1.1 and HTTP/2 support
- Connection pooling
- Automatic redirect following
- Cookie management
- Request/response streaming
- Compression (gzip, deflate, brotli)
- Custom headers and timeouts
- TLS/SSL with custom certificates

## Basic Usage

### Creating a Client

```php
$client = Client::new();

// Or with custom config
$client = Client::builder()
    ->timeout(30.0)              // 30 seconds timeout
    ->userAgent('MyApp/1.0')
    ->followRedirects(true)
    ->maxRedirects(10)
    ->build();
```

### GET Request

```php
$response = $client->get('https://httpbin.org/get');

echo "Status: {$response->status()}\n";
echo "Headers: " . json_encode($response->headers()) . "\n";
echo "Body: {$response->text()}\n";
```

### POST Request

```php
// JSON POST
$response = $client->post('https://httpbin.org/post', [
    'json' => [
        'name' => 'John',
        'email' => 'john@example.com'
    ]
]);

// Form POST
$response = $client->post('https://httpbin.org/post', [
    'form' => [
        'username' => 'john',
        'password' => 'secret'
    ]
]);

// Raw body POST
$response = $client->post('https://httpbin.org/post', [
    'body' => 'Raw request body',
    'headers' => [
        'Content-Type' => 'text/plain'
    ]
]);
```

### Other HTTP Methods

```php
// PUT
$response = $client->put('https://api.example.com/users/1', [
    'json' => ['name' => 'Updated Name']
]);

// PATCH
$response = $client->patch('https://api.example.com/users/1', [
    'json' => ['email' => 'new@example.com']
]);

// DELETE
$response = $client->delete('https://api.example.com/users/1');

// HEAD
$response = $client->head('https://api.example.com/users');

// OPTIONS
$response = $client->options('https://api.example.com/users');
```

## Response Handling

### Get Response Data

```php
$response = $client->get('https://httpbin.org/json');

// Get status code
$status = $response->status();

// Get headers
$headers = $response->headers();
$contentType = $response->header('Content-Type');

// Get body as text
$text = $response->text();

// Parse JSON
$data = $response->json();

// Get raw bytes
$bytes = $response->bytes();
```

### Check Response Status

```php
$response = $client->get('https://httpbin.org/status/200');

if ($response->isSuccess()) {
    echo "Success! Status: {$response->status()}\n";
}

if ($response->isRedirect()) {
    echo "Redirect to: {$response->header('Location')}\n";
}

if ($response->isClientError()) {
    echo "Client error: {$response->status()}\n";
}

if ($response->isServerError()) {
    echo "Server error: {$response->status()}\n";
}
```

## Advanced Features

### Custom Headers

```php
$response = $client->get('https://api.github.com/users/github', [
    'headers' => [
        'Authorization' => 'Bearer YOUR_TOKEN',
        'Accept' => 'application/vnd.github.v3+json',
        'X-Custom-Header' => 'value'
    ]
]);
```

### Query Parameters

```php
$response = $client->get('https://api.github.com/search/repositories', [
    'query' => [
        'q' => 'language:php',
        'sort' => 'stars',
        'order' => 'desc'
    ]
]);
```

### Timeout Configuration

```php
// Per-request timeout
$response = $client->get('https://slow-api.com/data', [
    'timeout' => 60.0  // 60 seconds
]);

// Connection timeout
$client = Client::builder()
    ->connectTimeout(10.0)  // 10 seconds to connect
    ->timeout(30.0)         // 30 seconds total
    ->build();
```

### Cookies

```php
// Automatic cookie jar
$client = Client::builder()
    ->cookieStore(true)
    ->build();

// Login
$client->post('https://example.com/login', [
    'form' => [
        'username' => 'user',
        'password' => 'pass'
    ]
]);

// Cookies are automatically sent in subsequent requests
$response = $client->get('https://example.com/dashboard');
```

### Redirects

```php
// Follow redirects (default: true)
$client = Client::builder()
    ->followRedirects(true)
    ->maxRedirects(10)
    ->build();

// Don't follow redirects
$client = Client::builder()
    ->followRedirects(false)
    ->build();
```

### Compression

```php
// Automatic compression (gzip, deflate, brotli)
// Enabled by default

// Disable compression
$client = Client::builder()
    ->compression(false)
    ->build();
```

### Custom TLS/SSL Certificates

```php
// Add custom CA certificate
$client = Client::builder()
    ->addRootCertificate('/path/to/ca-cert.pem')
    ->build();

// Accept invalid certificates (NOT recommended for production)
$client = Client::builder()
    ->dangerAcceptInvalidCerts(true)
    ->build();
```

## Concurrent Requests

### Using go() for Concurrency

```php
Kernel::run(function () {
    $client = Client::new();
    $results = [];

    $urls = [
        'https://api.github.com/users/github',
        'https://api.github.com/users/microsoft',
        'https://api.github.com/users/google',
    ];

    foreach ($urls as $url) {
        go(function() use ($client, $url, &$results) {
            $response = $client->get($url);
            $results[] = $response->json();
        });
    }

    // Wait for all requests to complete
    Time::sleep(2000);

    foreach ($results as $data) {
        echo "User: {$data['login']}\n";
    }
});
```

### With Error Handling

```php
$client = Client::new();

foreach ($urls as $url) {
    go(function() use ($client, $url, &$results, &$errors) {
        try {
            $response = $client->get($url);
            if ($response->isSuccess()) {
                $results[] = $response->json();
            } else {
                $errors[] = "HTTP {$response->status()} for $url";
            }
        } catch (Exception $e) {
            $errors[] = "Error fetching $url: {$e->getMessage()}";
        }
    });
}
```

## Streaming

### Download Large Files

```php
$client = Client::new();

// Stream response to file
$response = $client->get('https://example.com/large-file.zip');
$body = $response->bytes();
file_put_contents('/tmp/download.zip', $body);
```

### Upload Data

```php
// Upload file
$client->post('https://example.com/upload', [
    'body' => file_get_contents('/path/to/file.jpg'),
    'headers' => [
        'Content-Type' => 'image/jpeg'
    ]
]);
```

## Real-World Examples

### REST API Client

```php
class GitHubClient
{
    private Client $client;

    public function __construct(string $token)
    {
        $this->client = Client::builder()
            ->baseUrl('https://api.github.com')
            ->defaultHeaders([
                'Authorization' => "Bearer $token",
                'Accept' => 'application/vnd.github.v3+json'
            ])
            ->timeout(30.0)
            ->build();
    }

    public function getUser(string $username): array
    {
        $response = $this->client->get("/users/$username");
        return $response->json();
    }

    public function listRepos(string $username): array
    {
        $response = $this->client->get("/users/$username/repos", [
            'query' => ['sort' => 'updated']
        ]);
        return $response->json();
    }
}

// Usage
Kernel::run(function () {
    $github = new GitHubClient('your-token');
    $user = $github->getUser('github');
    echo "Name: {$user['name']}\n";
});
```

### Parallel API Calls

```php
function fetchUserData(int $userId): array
{
    $client = Client::new();
    $results = [];

    // Fetch from multiple endpoints concurrently
    go(function() use ($client, $userId, &$results) {
        $response = $client->get("https://api.example.com/users/$userId");
        $results['user'] = $response->json();
    });

    go(function() use ($client, $userId, &$results) {
        $response = $client->get("https://api.example.com/users/$userId/posts");
        $results['posts'] = $response->json();
    });

    go(function() use ($client, $userId, &$results) {
        $response = $client->get("https://api.example.com/users/$userId/comments");
        $results['comments'] = $response->json();
    });

    // Wait for all requests
    Time::sleep(1000);

    return $results;
}
```

## Best Practices

### 1. Reuse Client Instances
```php
// ✅ Good - Reuse client for connection pooling
$client = Client::new();
for ($i = 0; $i < 100; $i++) {
    $client->get("https://api.example.com/data/$i");
}

// ❌ Bad - Creates new connection pool each time
for ($i = 0; $i < 100; $i++) {
    $client = Client::new();
    $client->get("https://api.example.com/data/$i");
}
```

### 2. Set Appropriate Timeouts
```php
// ✅ Good
$client = Client::builder()
    ->connectTimeout(5.0)   // Fast fail for connection issues
    ->timeout(30.0)         // Reasonable total timeout
    ->build();
```

### 3. Handle Errors Gracefully
```php
try {
    $response = $client->get('https://api.example.com/data');
    if (!$response->isSuccess()) {
        throw new Exception("HTTP {$response->status()}");
    }
    $data = $response->json();
} catch (Exception $e) {
    logger()->error("API request failed: {$e->getMessage()}");
    // Handle error appropriately
}
```

### 4. Use Concurrency for Independent Requests
```php
// ✅ Good - Concurrent requests
foreach ($urls as $url) {
    go(fn() => $client->get($url));
}

// ❌ Bad - Sequential requests
foreach ($urls as $url) {
    $client->get($url);
}
```

## Next Steps

- [HTTP Server Tutorial](03-http-server.md) - Build async HTTP servers
- [Database Tutorial](04-pdo-database.md) - Async database operations
- [Async Patterns](06-async-patterns.md) - Advanced patterns
