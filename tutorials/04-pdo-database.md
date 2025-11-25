# PDO Database Tutorial

Use async PDO for high-performance database operations with MySQL and PostgreSQL.

## Quick Start

```php
use Async\Kernel;

Kernel::run(function () {
    $pdo = new \PDO\PDO(
        'mysql:host=localhost;dbname=test',
        'user',
        'password'
    );

    $stmt = $pdo->prepare('SELECT * FROM users WHERE id = ?');
    $stmt->execute([1]);
    $user = $stmt->fetch();

    echo "User: {$user['name']}\n";
});
```

## Features

- Full PDO compatibility
- Async operations (non-blocking)
- Connection pooling
- MySQL and PostgreSQL support
- Prepared statements with parameter binding
- Transactions
- Error handling
- Zero-copy result fetching

## Connecting to Database

### MySQL Connection

```php
$pdo = new \PDO\PDO(
    'mysql:host=localhost;port=3306;dbname=mydb',
    'username',
    'password',
    [
        \PDO\PDO::ATTR_ERRMODE => \PDO\PDO::ERRMODE_EXCEPTION,
        \PDO\PDO::ATTR_DEFAULT_FETCH_MODE => \PDO\PDO::FETCH_ASSOC,
    ]
);
```

### PostgreSQL Connection

```php
$pdo = new \PDO\PDO(
    'pgsql:host=localhost;port=5432;dbname=mydb',
    'username',
    'password',
    [
        \PDO\PDO::ATTR_ERRMODE => \PDO\PDO::ERRMODE_EXCEPTION,
    ]
);
```

### Connection Pool Options

```php
$pdo = new \PDO\PDO(
    'mysql:host=localhost;dbname=test',
    'user',
    'pass',
    [
        // Connection pool settings
        \PDO\PDO::ATTR_CONNECTION_POOL_SIZE => 10,      // Max connections
        \PDO\PDO::ATTR_CONNECTION_POOL_MIN_SIZE => 2,   // Min connections
        \PDO\PDO::ATTR_CONNECTION_POOL_WAIT_TIMEOUT => 30.0,  // Wait timeout
        \PDO\PDO::ATTR_CONNECTION_POOL_IDLE_TIME => 600.0,    // Idle timeout
        \PDO\PDO::ATTR_CONNECTION_POOL_HEARTBEAT => true,     // Heartbeat
    ]
);
```

## Basic Queries

### SELECT Query

```php
// Query without parameters
$stmt = $pdo->query('SELECT * FROM users');
$users = $stmt->fetchAll();

foreach ($users as $user) {
    echo "User: {$user['name']}\n";
}
```

### INSERT Query

```php
// Simple insert
$affected = $pdo->exec("INSERT INTO users (name, email) VALUES ('Alice', 'alice@example.com')");
echo "Inserted $affected row(s)\n";

// Get last insert ID
$id = $pdo->lastInsertId();
echo "New user ID: $id\n";
```

### UPDATE Query

```php
$affected = $pdo->exec("UPDATE users SET email = 'newemail@example.com' WHERE id = 1");
echo "Updated $affected row(s)\n";
```

### DELETE Query

```php
$affected = $pdo->exec("DELETE FROM users WHERE id = 1");
echo "Deleted $affected row(s)\n";
```

## Prepared Statements

### Positional Parameters

```php
// Prepare statement
$stmt = $pdo->prepare('SELECT * FROM users WHERE id = ?');

// Execute with parameters
$stmt->execute([1]);

// Fetch result
$user = $stmt->fetch();
```

### Named Parameters

```php
// Prepare with named parameters
$stmt = $pdo->prepare('SELECT * FROM users WHERE name = :name AND email = :email');

// Execute with named parameters
$stmt->execute([
    ':name' => 'Alice',
    ':email' => 'alice@example.com'
]);

// Or without colons
$stmt->execute([
    'name' => 'Alice',
    'email' => 'alice@example.com'
]);

$user = $stmt->fetch();
```

### Multiple Executions

```php
$stmt = $pdo->prepare('INSERT INTO users (name, email) VALUES (?, ?)');

$users = [
    ['Alice', 'alice@example.com'],
    ['Bob', 'bob@example.com'],
    ['Charlie', 'charlie@example.com'],
];

foreach ($users as $user) {
    $stmt->execute($user);
}
```

## Parameter Binding

### bindValue()

```php
$stmt = $pdo->prepare('SELECT * FROM users WHERE age > ? AND city = ?');

$stmt->bindValue(1, 18, \PDO\PDO::PARAM_INT);
$stmt->bindValue(2, 'New York', \PDO\PDO::PARAM_STR);

$stmt->execute();
$users = $stmt->fetchAll();
```

### bindParam()

```php
$stmt = $pdo->prepare('INSERT INTO users (name, age) VALUES (?, ?)');

$name = '';
$age = 0;

$stmt->bindParam(1, $name, \PDO\PDO::PARAM_STR);
$stmt->bindParam(2, $age, \PDO\PDO::PARAM_INT);

// Change values and execute multiple times
$users = [
    ['Alice', 25],
    ['Bob', 30],
    ['Charlie', 35],
];

foreach ($users as [$n, $a]) {
    $name = $n;
    $age = $a;
    $stmt->execute();
}
```

### Parameter Types

```php
// Integer parameter
$stmt->bindValue(1, 42, \PDO\PDO::PARAM_INT);

// String parameter (default)
$stmt->bindValue(2, 'text', \PDO\PDO::PARAM_STR);

// Boolean parameter
$stmt->bindValue(3, true, \PDO\PDO::PARAM_BOOL);

// NULL parameter
$stmt->bindValue(4, null, \PDO\PDO::PARAM_NULL);

// LOB (Large Object) parameter
$fp = fopen('/path/to/file', 'rb');
$stmt->bindValue(5, $fp, \PDO\PDO::PARAM_LOB);
```

## Fetching Results

### fetch() - Single Row

```php
$stmt = $pdo->query('SELECT * FROM users LIMIT 1');

// Fetch as associative array (default)
$user = $stmt->fetch(\PDO\PDO::FETCH_ASSOC);

// Fetch as indexed array
$user = $stmt->fetch(\PDO\PDO::FETCH_NUM);

// Fetch as both
$user = $stmt->fetch(\PDO\PDO::FETCH_BOTH);

// Fetch as object
$user = $stmt->fetch(\PDO\PDO::FETCH_OBJ);
```

### fetchAll() - All Rows

```php
$stmt = $pdo->query('SELECT * FROM users');

// Fetch all as associative arrays
$users = $stmt->fetchAll(\PDO\PDO::FETCH_ASSOC);

// Fetch all as objects
$users = $stmt->fetchAll(\PDO\PDO::FETCH_OBJ);

// Fetch all as custom class
$users = $stmt->fetchAll(\PDO\PDO::FETCH_CLASS, 'User');
```

### fetchColumn() - Single Column

```php
$stmt = $pdo->query('SELECT COUNT(*) FROM users');
$count = $stmt->fetchColumn();

echo "Total users: $count\n";
```

### Fetch Modes

```php
// Set default fetch mode
$pdo->setAttribute(\PDO\PDO::ATTR_DEFAULT_FETCH_MODE, \PDO\PDO::FETCH_ASSOC);

// Fetch into specific class
class User {
    public int $id;
    public string $name;
    public string $email;
}

$stmt = $pdo->query('SELECT * FROM users');
$users = $stmt->fetchAll(\PDO\PDO::FETCH_CLASS, 'User');
```

## Transactions

### Basic Transaction

```php
try {
    $pdo->beginTransaction();

    $pdo->exec("INSERT INTO users (name) VALUES ('Alice')");
    $pdo->exec("INSERT INTO users (name) VALUES ('Bob')");

    $pdo->commit();
    echo "Transaction committed\n";
} catch (Exception $e) {
    $pdo->rollBack();
    echo "Transaction rolled back: {$e->getMessage()}\n";
}
```

### Nested Operations

```php
$pdo->beginTransaction();

try {
    // Insert user
    $stmt = $pdo->prepare('INSERT INTO users (name, email) VALUES (?, ?)');
    $stmt->execute(['Alice', 'alice@example.com']);
    $userId = $pdo->lastInsertId();

    // Insert user profile
    $stmt = $pdo->prepare('INSERT INTO profiles (user_id, bio) VALUES (?, ?)');
    $stmt->execute([$userId, 'Alice bio']);

    $pdo->commit();
} catch (Exception $e) {
    $pdo->rollBack();
    throw $e;
}
```

### Check Transaction Status

```php
if ($pdo->inTransaction()) {
    echo "Currently in transaction\n";
} else {
    echo "Not in transaction\n";
}
```

## Error Handling

### Exception Mode

```php
$pdo = new \PDO\PDO($dsn, $user, $pass, [
    \PDO\PDO::ATTR_ERRMODE => \PDO\PDO::ERRMODE_EXCEPTION
]);

try {
    $stmt = $pdo->query('SELECT * FROM non_existent_table');
} catch (\PDO\PDOException $e) {
    echo "Database error: {$e->getMessage()}\n";
    echo "Error code: {$e->getCode()}\n";
}
```

### Silent Mode with Error Checking

```php
$pdo = new \PDO\PDO($dsn, $user, $pass, [
    \PDO\PDO::ATTR_ERRMODE => \PDO\PDO::ERRMODE_SILENT
]);

$stmt = $pdo->prepare('SELECT * FROM users WHERE id = ?');
$success = $stmt->execute([1]);

if (!$success) {
    $errorInfo = $stmt->errorInfo();
    echo "Error: {$errorInfo[2]}\n"; // Error message
    echo "SQLSTATE: {$errorInfo[0]}\n";
    echo "Driver code: {$errorInfo[1]}\n";
}
```

## Real-World Examples

### User CRUD Operations

```php
class UserRepository
{
    private \PDO\PDO $pdo;

    public function __construct(\PDO\PDO $pdo)
    {
        $this->pdo = $pdo;
    }

    public function find(int $id): ?array
    {
        $stmt = $this->pdo->prepare('SELECT * FROM users WHERE id = ?');
        $stmt->execute([$id]);
        $user = $stmt->fetch(\PDO\PDO::FETCH_ASSOC);
        return $user ?: null;
    }

    public function findAll(): array
    {
        $stmt = $this->pdo->query('SELECT * FROM users ORDER BY created_at DESC');
        return $stmt->fetchAll(\PDO\PDO::FETCH_ASSOC);
    }

    public function create(string $name, string $email): int
    {
        $stmt = $this->pdo->prepare(
            'INSERT INTO users (name, email, created_at) VALUES (?, ?, NOW())'
        );
        $stmt->execute([$name, $email]);
        return (int)$this->pdo->lastInsertId();
    }

    public function update(int $id, array $data): bool
    {
        $fields = [];
        $values = [];

        foreach ($data as $key => $value) {
            $fields[] = "$key = ?";
            $values[] = $value;
        }

        $values[] = $id;

        $sql = 'UPDATE users SET ' . implode(', ', $fields) . ' WHERE id = ?';
        $stmt = $this->pdo->prepare($sql);
        $stmt->execute($values);

        return $stmt->rowCount() > 0;
    }

    public function delete(int $id): bool
    {
        $stmt = $this->pdo->prepare('DELETE FROM users WHERE id = ?');
        $stmt->execute([$id]);
        return $stmt->rowCount() > 0;
    }
}

// Usage
Kernel::run(function () {
    $pdo = new \PDO\PDO('mysql:host=localhost;dbname=test', 'user', 'pass');
    $users = new UserRepository($pdo);

    // Create user
    $id = $users->create('Alice', 'alice@example.com');
    echo "Created user with ID: $id\n";

    // Find user
    $user = $users->find($id);
    echo "Found user: {$user['name']}\n";

    // Update user
    $users->update($id, ['name' => 'Alice Smith']);

    // Delete user
    $users->delete($id);
});
```

### Concurrent Database Queries

```php
Kernel::run(function () {
    $pdo = new \PDO\PDO('mysql:host=localhost;dbname=test', 'user', 'pass');

    $results = [];

    // Execute multiple queries concurrently
    go(function() use ($pdo, &$results) {
        $stmt = $pdo->query('SELECT COUNT(*) FROM users');
        $results['user_count'] = $stmt->fetchColumn();
    });

    go(function() use ($pdo, &$results) {
        $stmt = $pdo->query('SELECT COUNT(*) FROM posts');
        $results['post_count'] = $stmt->fetchColumn();
    });

    go(function() use ($pdo, &$results) {
        $stmt = $pdo->query('SELECT COUNT(*) FROM comments');
        $results['comment_count'] = $stmt->fetchColumn();
    });

    // Wait for all queries
    Time::sleep(100);

    echo "Users: {$results['user_count']}\n";
    echo "Posts: {$results['post_count']}\n";
    echo "Comments: {$results['comment_count']}\n";
});
```

### HTTP API with Database

```php
Kernel::run(function () {
    $pdo = new \PDO\PDO('mysql:host=localhost;dbname=test', 'user', 'pass', [
        \PDO\PDO::ATTR_CONNECTION_POOL_SIZE => 20
    ]);

    $server = new Server();
    $listener = Listener::bind('127.0.0.1:8080');

    while (true) {
        $conn = $listener->accept();

        go(function() use ($server, $conn, $pdo) {
            $server->serve($conn, function(HttpRequest $req) use ($pdo): HttpResponse {
                $path = $req->path();
                $method = $req->method();
                $resp = new HttpResponse();

                // GET /api/users
                if ($method === 'GET' && $path === '/api/users') {
                    $stmt = $pdo->query('SELECT id, name, email FROM users');
                    $users = $stmt->fetchAll(\PDO\PDO::FETCH_ASSOC);

                    $resp->setStatus(200);
                    $resp->setHeader('Content-Type', 'application/json');
                    $resp->setBody(json_encode($users));
                }
                // GET /api/users/:id
                elseif ($method === 'GET' && preg_match('#^/api/users/(\d+)$#', $path, $m)) {
                    $id = $m[1];
                    $stmt = $pdo->prepare('SELECT id, name, email FROM users WHERE id = ?');
                    $stmt->execute([$id]);
                    $user = $stmt->fetch(\PDO\PDO::FETCH_ASSOC);

                    if ($user) {
                        $resp->setStatus(200);
                        $resp->setHeader('Content-Type', 'application/json');
                        $resp->setBody(json_encode($user));
                    } else {
                        $resp->setStatus(404);
                        $resp->setBody(json_encode(['error' => 'User not found']));
                    }
                }
                // POST /api/users
                elseif ($method === 'POST' && $path === '/api/users') {
                    $data = json_decode($req->body(), true);
                    $stmt = $pdo->prepare('INSERT INTO users (name, email) VALUES (?, ?)');
                    $stmt->execute([$data['name'], $data['email']]);

                    $id = $pdo->lastInsertId();

                    $resp->setStatus(201);
                    $resp->setHeader('Content-Type', 'application/json');
                    $resp->setBody(json_encode(['id' => $id]));
                }
                else {
                    $resp->setStatus(404);
                    $resp->setBody(json_encode(['error' => 'Not Found']));
                }

                return $resp;
            });
        });
    }
});
```

## PostgreSQL-Specific Features

### RETURNING Clause

```php
$stmt = $pdo->prepare('INSERT INTO users (name, email) VALUES (?, ?) RETURNING id');
$stmt->execute(['Alice', 'alice@example.com']);
$id = $stmt->fetchColumn();
```

### Arrays

```php
// Insert array
$stmt = $pdo->prepare("INSERT INTO posts (tags) VALUES (?)");
$stmt->execute(['{php,async,database}']);

// Query array
$stmt = $pdo->query("SELECT * FROM posts WHERE 'php' = ANY(tags)");
```

### JSON/JSONB

```php
// Insert JSON
$stmt = $pdo->prepare("INSERT INTO users (metadata) VALUES (?)");
$stmt->execute([json_encode(['role' => 'admin'])]);

// Query JSON
$stmt = $pdo->query("SELECT * FROM users WHERE metadata->>'role' = 'admin'");
```

## Best Practices

### 1. Use Connection Pooling
```php
$pdo = new \PDO\PDO($dsn, $user, $pass, [
    \PDO\PDO::ATTR_CONNECTION_POOL_SIZE => 20  // Reuse connections
]);
```

### 2. Always Use Prepared Statements
```php
// ✅ Good - Safe from SQL injection
$stmt = $pdo->prepare('SELECT * FROM users WHERE email = ?');
$stmt->execute([$email]);

// ❌ Bad - SQL injection risk
$pdo->query("SELECT * FROM users WHERE email = '$email'");
```

### 3. Use Transactions for Multiple Writes
```php
$pdo->beginTransaction();
try {
    // Multiple inserts/updates
    $pdo->commit();
} catch (Exception $e) {
    $pdo->rollBack();
}
```

### 4. Handle Errors Appropriately
```php
$pdo = new \PDO\PDO($dsn, $user, $pass, [
    \PDO\PDO::ATTR_ERRMODE => \PDO\PDO::ERRMODE_EXCEPTION
]);

try {
    // Database operations
} catch (\PDO\PDOException $e) {
    logger()->error("Database error: {$e->getMessage()}");
}
```

## Next Steps

- [Network Programming](05-network.md) - TCP, UDP, TLS
- [Async Patterns](06-async-patterns.md) - Advanced async patterns
- [HTTP Server](03-http-server.md) - Build APIs with databases
