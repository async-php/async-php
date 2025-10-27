<?php

namespace Async\Stream;

use Async\Kernel\Network\TcpStream;
use Async\Kernel\Network\TlsStream;
use Fiber;

class HttpStreamWrapper
{
    /** @var resource|null */
    public $context;

    private ?object $conn = null; // TcpStream|TlsStream
    private string $body = '';
    private int $position = 0;
    private bool $eof = false;

    private array $responseHeaders = [];
    private int $statusCode = 0;

    public function stream_open(string $path, string $mode, int $options, ?string &$opened_path): bool
    {
        $parts = parse_url($path);
        if (!$parts || !isset($parts['host'])) {
            return false;
        }

        $scheme = $parts['scheme'] ?? 'http';
        $host = $parts['host'];
        $port = $parts['port'] ?? ($scheme === 'https' ? 443 : 80);
        $target = ($parts['path'] ?? '/') . (isset($parts['query']) ? '?' . $parts['query'] : '');

        $contextOpts = $this->context ? stream_context_get_options($this->context) : [];
        $httpOpts = $contextOpts['http'] ?? [];

        $method = strtoupper($httpOpts['method'] ?? 'GET');
        $content = (string)($httpOpts['content'] ?? '');
        $protocolVersion = $httpOpts['protocol_version'] ?? '1.1';

        $response = $this->sendRequest($scheme, $host, $port, $target, $httpOpts, $method, $protocolVersion, $content);
        if ($response === false) {
            return false;
        }

        $this->statusCode = $response['status'];
        $this->responseHeaders = $response['headers'];
        $this->body = $response['body'];
        $this->position = 0;
        $this->eof = ($this->body === '');

        if ($opened_path !== null) {
            $opened_path = $path;
        }

        return true;
    }

    public function stream_read(int $count): string|false
    {
        if ($this->eof) return '';

        $chunk = substr($this->body, $this->position, $count);
        $this->position += strlen($chunk);

        if ($this->position >= strlen($this->body)) {
            $this->eof = true;
        }

        return $chunk;
    }

    public function stream_eof(): bool
    {
        return $this->eof;
    }

    public function stream_stat(): array|false
    {
        return false;
    }

    public function url_stat(string $path, int $flags): array|false
    {
        $parts = parse_url($path);
        if (!$parts || !isset($parts['host'])) {
            return ($flags & STREAM_URL_STAT_QUIET) ? false : false;
        }

        $scheme = $parts['scheme'] ?? 'http';
        $host = $parts['host'];
        $port = $parts['port'] ?? ($scheme === 'https' ? 443 : 80);
        $target = ($parts['path'] ?? '/') . (isset($parts['query']) ? '?' . $parts['query'] : '');

        $contextOpts = $this->context ? stream_context_get_options($this->context) : [];
        $httpOpts = $contextOpts['http'] ?? [];
        $protocolVersion = $httpOpts['protocol_version'] ?? '1.1';

        $response = $this->sendRequest($scheme, $host, $port, $target, $httpOpts, 'HEAD', $protocolVersion, '');
        if ($response === false || $response['status'] >= 400) {
            return false;
        }

        $size = 0;
        if (isset($response['headers']['content-length'])) {
            $size = (int)$response['headers']['content-length'];
        }

        $mode = 0100000; // regular file
        return [
            'dev' => 0, 'ino' => 0, 'mode' => $mode, 'nlink' => 1,
            'uid' => 0, 'gid' => 0, 'rdev' => 0, 'size' => $size,
            'atime' => 0, 'mtime' => 0, 'ctime' => 0, 'blksize' => 4096, 'blocks' => 1,
        ];
    }

    private function decodeChunkedBody(string $data): string
    {
        $result = '';
        $offset = 0;
        $len = strlen($data);

        while ($offset < $len) {
            $newlinePos = strpos($data, "\r\n", $offset);
            if ($newlinePos === false) {
                break;
            }
            $line = substr($data, $offset, $newlinePos - $offset);
            $chunkLen = hexdec(trim($line));
            $offset = $newlinePos + 2;

            if ($chunkLen === 0) {
                break;
            }

            $result .= substr($data, $offset, $chunkLen);
            $offset += $chunkLen + 2; // skip chunk + CRLF
        }

        return $result;
    }

    private function sendRequest(string $scheme, string $host, int $port, string $target, array $httpOpts, string $method, string $protocolVersion, string $content): array|false
    {
        $userHeaders = $httpOpts['header'] ?? [];
        if (is_string($userHeaders)) {
            $userHeaders = preg_split('/\r?\n/', $userHeaders, -1, PREG_SPLIT_NO_EMPTY);
        }
        if (!is_array($userHeaders)) {
            $userHeaders = [];
        }

        $headers = [];
        $headers[] = "Host: {$host}" . (($scheme === 'https' && $port === 443) || ($scheme === 'http' && $port === 80) ? '' : ":{$port}");
        $headers[] = "Connection: close";

        $hasContentLength = false;
        foreach ($userHeaders as $line) {
            $trimmed = trim($line);
            if ($trimmed === '') continue;
            if (stripos($trimmed, 'content-length:') === 0) {
                $hasContentLength = true;
            }
            $headers[] = $trimmed;
        }

        if ($content !== '' && !$hasContentLength) {
            $headers[] = 'Content-Length: ' . strlen($content);
        }

        $request = sprintf("%s %s HTTP/%s\r\n%s\r\n\r\n%s", $method, $target, $protocolVersion, implode("\r\n", $headers), $content);

        if ($scheme === 'https') {
            $this->conn = Fiber::suspend(TlsStream::connect($host, $port, null));
        } else {
            $addr = $host . ':' . $port;
            $this->conn = Fiber::suspend(TcpStream::connect($addr));
        }

        if (!$this->conn) {
            return false;
        }

        $written = Fiber::suspend($this->conn->write($request));
        if ($written === false) {
            Fiber::suspend($this->conn->close());
            $this->conn = null;
            return false;
        }

        $response = '';
        while (true) {
            $chunk = Fiber::suspend($this->conn->read(4096));
            if ($chunk === false || $chunk === null || $chunk === '') {
                break;
            }
            $response .= $chunk;
        }

        Fiber::suspend($this->conn->close());
        $this->conn = null;

        [$headerBlock, $body] = array_pad(explode("\r\n\r\n", $response, 2), 2, '');

        $lines = preg_split('/\r?\n/', $headerBlock);
        $statusLine = array_shift($lines);
        $statusCode = 0;
        if ($statusLine && preg_match('#HTTP/[0-9.]+\s+(\d{3})#', $statusLine, $m)) {
            $statusCode = (int)$m[1];
        }

        $headersAssoc = [];
        foreach ($lines as $line) {
            if (strpos($line, ':') !== false) {
                [$k, $v] = explode(':', $line, 2);
                $headersAssoc[strtolower(trim($k))] = trim($v);
            }
        }

        if (($headersAssoc['transfer-encoding'] ?? '') === 'chunked') {
            $body = $this->decodeChunkedBody($body);
        } elseif (isset($headersAssoc['content-length'])) {
            $len = (int)$headersAssoc['content-length'];
            $body = substr($body, 0, $len);
        }

        return [
            'status' => $statusCode,
            'headers' => $headersAssoc,
            'body' => $body,
        ];
    }
}
