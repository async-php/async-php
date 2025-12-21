# HTTP/3 and QUIC Examples

This directory contains examples demonstrating HTTP/3 and QUIC functionality in Async-PHP.

## Prerequisites

1. **Generate SSL/TLS Certificates**
   ```bash
   cd examples/http3
   ./generate-certs.sh
   ```

   This creates self-signed certificates in `examples/http3/certs/`:
   - `server.crt` - Server certificate
   - `server.key` - Private key

   ⚠️ These are for testing only! Do not use in production.

2. **Build the Extension**
   ```bash
   cargo build --release
   ```

## Examples

### 1. Unified Server (Recommended)

**File:** `unified_server.php`

Runs HTTP/1, HTTP/2, and HTTP/3 simultaneously on different ports.

```bash
php -d extension=target/release/libasync_php.dylib examples/http3/unified_server.php
```

**Endpoints:**
- `http://localhost:8080/` - HTTP/1.1, HTTP/2 (plain)
- `https://localhost:8443/` - HTTP/1.1, HTTP/2 (TLS)
- `https://localhost:4433/` - HTTP/3 (QUIC)

**Test:**
```bash
# HTTP/1.1
curl http://localhost:8080/hello

# HTTP/2 (requires curl with HTTP/2 support)
curl --http2 -k https://localhost:8443/hello

# HTTP/3 (requires curl with HTTP/3 support)
curl --http3 -k https://localhost:4433/hello
```

### 2. QUIC Echo Server

**File:** `quic_echo_server.php`

Simple QUIC server that echoes back any data received.

```bash
php -d extension=target/release/libasync_php.dylib examples/http3/quic_echo_server.php
```

### 3. QUIC Client

**File:** `quic_client.php`

Demonstrates QUIC client functionality: connecting, sending/receiving data over streams.

**Usage:**
1. Start the echo server first:
   ```bash
   php -d extension=target/release/libasync_php.dylib examples/http3/quic_echo_server.php
   ```

2. In another terminal, run the client:
   ```bash
   php -d extension=target/release/libasync_php.dylib examples/http3/quic_client.php
   ```

### 4. HTTP/3 Server (Standalone)

**File:** `server.php`

HTTP/3-only server.

```bash
php -d extension=target/release/libasync_php.dylib examples/http3/server.php
```

## Protocol Features

### HTTP/3 (QUIC)
- Built-in TLS 1.3 encryption
- Multiplexed streams without head-of-line blocking
- 0-RTT connection resumption
- Connection migration
- Better performance on lossy networks

### HTTP/2
- Multiplexed streams
- Header compression (HPACK)
- Server push support
- Stream prioritization

### HTTP/1.1
- Simple request/response model
- Widely supported
- Good for simple use cases

## Configuration Examples

### Simple HTTP Server
```php
use Async\Network\Http\Server;

Server::listen('0.0.0.0:8080', function($req) {
    $resp = new Response();
    $resp->withBody("Hello World!");
    return $resp;
});
```

### Multi-Protocol Server
```php
use Async\Network\Http\Server;

$server = new Server();
$server->listenAndServe([
    'http' => '0.0.0.0:8080',      // HTTP/1, HTTP/2
    'https' => [                    // HTTP/1, HTTP/2 (TLS)
        'addr' => '0.0.0.0:8443',
        'cert' => 'cert.pem',
        'key' => 'key.pem'
    ],
    'http3' => [                    // HTTP/3 (QUIC)
        'addr' => '0.0.0.0:4433',
        'cert' => 'cert.pem',
        'key' => 'key.pem'
    ]
], $handler);
```

### QUIC Direct Usage
```php
use Async\Network\Quic\Listener;
use Async\Network\Quic\Connection;

// Server
$listener = Listener::bind('0.0.0.0:4433', [
    'cert_path' => 'cert.pem',
    'key_path' => 'key.pem'
]);

while (true) {
    $conn = $listener->accept();
    go(function() use ($conn) {
        $stream = $conn->acceptBiStream();
        $data = $stream->read(1024);
        $stream->write("Echo: $data");
        $stream->finish();
    });
}

// Client
$conn = Connection::connect('127.0.0.1:4433', 'localhost', [
    'verify_cert' => false
]);

$stream = $conn->openBiStream();
$stream->write("Hello QUIC!");
$stream->finish();
$response = $stream->read(1024);
echo $response;
```

## Testing Tools

### curl with HTTP/3
To test HTTP/3, you need curl compiled with HTTP/3 support:

**macOS (Homebrew):**
```bash
brew install curl --with-quiche
```

**Ubuntu/Debian:**
```bash
# Build from source with HTTP/3 support
# See: https://github.com/curl/curl/blob/master/docs/HTTP3.md
```

**Test HTTP/3:**
```bash
curl --http3 -k https://localhost:4433/
```

### Chrome/Chromium
Chrome has built-in HTTP/3 support:
1. Open `chrome://flags`
2. Enable "Experimental QUIC protocol"
3. Visit your HTTP/3 server

### Firefox
Firefox supports HTTP/3:
1. Open `about:config`
2. Set `network.http.http3.enabled` to `true`
3. Visit your HTTP/3 server

## Troubleshooting

### Certificate Errors
If you see certificate errors, make sure you're using the `-k` flag with curl to skip verification (for testing only):
```bash
curl -k https://localhost:4433/
```

### Connection Refused
- Make sure the server is running
- Check if the port is already in use: `lsof -i :4433`
- Verify firewall settings

### HTTP/3 Not Working
- Ensure curl has HTTP/3 support: `curl --version | grep HTTP3`
- Try HTTP/2 first to verify TLS is working
- Check server logs for errors

## Performance Tips

1. **Use HTTP/3 for mobile/unstable networks** - Better handling of packet loss
2. **Use HTTP/2 for stable networks** - Lower overhead
3. **Enable connection pooling** - Reuse connections when possible
4. **Optimize certificate size** - Smaller certificates = faster handshakes

## Security Notes

⚠️ **Important:**
- Never use self-signed certificates in production
- Always verify certificates in production (set `verify_cert: true`)
- Use proper CA-signed certificates
- Keep your private keys secure
- Regularly update TLS libraries

## Resources

- [HTTP/3 Specification](https://datatracker.ietf.org/doc/html/rfc9114)
- [QUIC Specification](https://datatracker.ietf.org/doc/html/rfc9000)
- [Quinn Rust Library](https://github.com/quinn-rs/quinn)
- [curl HTTP/3 Guide](https://github.com/curl/curl/blob/master/docs/HTTP3.md)
