# Development Notes

## Debug vs Release Builds

### **⚠️ CRITICAL: Always Use Release Builds for Testing**

Debug builds have severe performance issues that make them unsuitable for most development work.

### TLS/HTTPS Performance Issue

**Problem**: The `ring` cryptography library used by `rustls` is **100-1000x slower** in debug builds.

**Impact**:
- ✅ HTTP requests (no encryption): Work fine in debug builds
- ❌ HTTPS requests (TLS/SSL): **Hang or timeout** in debug builds (can take minutes)
- ✅ HTTPS requests: Work perfectly in release builds

**Root Cause**:
Cryptographic operations in `ring` are heavily optimized. Without compiler optimizations, operations like:
- TLS handshakes
- Certificate verification
- Encryption/decryption

become prohibitively slow, often causing apparent hangs or timeouts.

### Recommended Workflow

**For Development and Testing:**
```bash
# Build release version (RECOMMENDED)
cargo build --release

# Use release extension
php -d extension=target/release/libasync_php.dylib your_script.php
```

**Debug Builds Only For:**
```bash
# Build debug version
cargo build

# Only use for:
# 1. Non-TLS HTTP testing
# 2. Debugging non-crypto code paths
# 3. Running with lldb/gdb (be patient with TLS!)
php -d extension=target/debug/libasync_php.dylib test_http_no_tls.php
```

### Test Examples

```php
// ✅ Works in both debug and release
$client = new HttpClient();
$response = $client->get('http://example.com')->send();  // HTTP, no TLS

// ❌ Hangs in debug, works in release
$client = new HttpClient();
$response = $client->get('https://httpbin.org/get')->send();  // HTTPS with TLS
```

### Performance Comparison

| Operation | Debug Build | Release Build |
|-----------|-------------|---------------|
| HTTP request | ~100ms | ~100ms |
| TLS handshake | **60+ seconds** (often timeout) | ~200ms |
| AES encryption | ~1000x slower | Fast |
| SHA256 hash | ~500x slower | Fast |

### Default Timeouts

To prevent indefinite hangs, HttpClient now has default timeouts:
- **Request timeout**: 30 seconds (total request time)
- **Connect timeout**: 10 seconds (connection establishment)

Override if needed:
```php
// timeout, connect_timeout, pool_idle_timeout, pool_max_idle, max_redirects
$client = new HttpClient(60.0, 30.0, null, null, null);
```

### Debugging TLS Issues

If you must debug TLS code:

1. Use release build for faster iteration
2. Or add extensive logging and be very patient
3. Or test TLS-specific code with minimal crypto (test vectors)

### Alternative: Profile-Optimized Debug Builds

If you need debug symbols but better performance:

```toml
# In Cargo.toml
[profile.dev]
opt-level = 1  # Basic optimizations
```

Or use the `dev` profile with optimization for specific dependencies:

```toml
[profile.dev.package."ring"]
opt-level = 3
```

This won't help much as the issue is systemic in unoptimized crypto code.

## Summary

**Use release builds unless you have a specific reason to use debug builds and are willing to work around the TLS performance issue.**

```bash
# This is your friend
cargo build --release && php -d extension=target/release/libasync_php.dylib test.php
```
