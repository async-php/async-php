<?php

namespace Async\Stream;

use Async\Kernel\Network\Http\Client as HttpClient;
use Fiber;

class HttpStreamWrapper
{
    /** @var resource|null */
    public $context;

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

        $contextOpts = $this->context ? stream_context_get_options($this->context) : [];
        $httpOpts = $contextOpts['http'] ?? [];

        $method = strtoupper($httpOpts['method'] ?? 'GET');
        $content = (string)($httpOpts['content'] ?? '');
        $headersOpt = $this->normalizeHeaders($httpOpts['header'] ?? []);
        $response = Fiber::suspend(HttpClient::request($method, $path, $headersOpt, $content));
        if (!is_array($response)) {
            return false;
        }

        $this->statusCode = (int)($response['status'] ?? 0);
        $this->responseHeaders = $response['headers'] ?? [];
        $this->body = (string)($response['body'] ?? '');
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

        $contextOpts = $this->context ? stream_context_get_options($this->context) : [];
        $httpOpts = $contextOpts['http'] ?? [];

        $headersOpt = $this->normalizeHeaders($httpOpts['header'] ?? []);
        $response = Fiber::suspend(HttpClient::request('HEAD', $path, $headersOpt, ''));
        if (!is_array($response) || ($response['status'] ?? 0) >= 400) {
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

    private function normalizeHeaders(array|string $headers): array
    {
        if (is_string($headers)) {
            $headers = preg_split('/\r?\n/', $headers, -1, PREG_SPLIT_NO_EMPTY) ?: [];
        }

        $normalized = [];
        foreach ($headers as $line) {
            if (strpos($line, ':') !== false) {
                [$k, $v] = explode(':', $line, 2);
                $normalized[trim($k)] = trim($v);
            }
        }

        return $normalized;
    }
}
