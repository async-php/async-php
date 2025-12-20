<?php

namespace Async\FileSystem;

use Async\IO\Reader;
use Async\IO\Writer;
use Async\IO\Closer;
use Async\IO\Seeker;
use Async\IO\ReaderAt;
use Async\IO\WriterAt;
use Async\IO\ReaderFrom;
use Async\IO\WriterTo;
use Async\IO;
use Async\Kernel\FileSystem\FileHandle as KernelFileHandle;
use Fiber;

/**
 * FileHandle represents an open file
 * Implements IO interfaces directly
 */
class FileHandle implements Reader, Writer, Closer, Seeker, ReaderAt, WriterAt, ReaderFrom, WriterTo
{
    private KernelFileHandle $inner;

    public function __construct(KernelFileHandle $inner)
    {
        $this->inner = $inner;
    }

    public static function open(string $path, string $mode): self
    {
        $future = KernelFileHandle::open($path, $mode);
        $kernelHandle = Fiber::suspend($future);
        if (!$kernelHandle) {
            throw new \RuntimeException("Failed to open file $path");
        }
        return new self($kernelHandle);
    }

    public function read(int $length): ?string
    {
        $future = $this->inner->read($length);
        $result = Fiber::suspend($future);
        return ($result === false || $result === '') ? null : $result;
    }

    public function write(string $data): int
    {
        $future = $this->inner->write($data);
        $result = Fiber::suspend($future);
        return $result === false ? 0 : (int)$result;
    }

    public function flush(): void
    {
        $future = $this->inner->flush();
        Fiber::suspend($future);
    }

    /**
     * Sync all data and metadata to disk (like fsync)
     */
    public function syncAll(): bool
    {
        $future = $this->inner->sync_all();
        return (bool)Fiber::suspend($future);
    }

    /**
     * Sync only data to disk, not metadata (like fdatasync)
     */
    public function syncData(): bool
    {
        $future = $this->inner->sync_data();
        return (bool)Fiber::suspend($future);
    }

    public function seek(int $offset, int $whence = self::SEEK_START): int
    {
        $future = $this->inner->seek($offset, $whence);
        $result = Fiber::suspend($future);

        if ($result === false) {
            throw new \RuntimeException("Seek operation failed");
        }

        return (int)$result;
    }

    public function close(): bool
    {
        $future = $this->inner->close();
        return (bool)Fiber::suspend($future);
    }

    public function readAt(int $offset, int $length): ?string
    {
        // Save current position
        $currentPos = $this->seek(0, self::SEEK_CURRENT);

        // Seek to target offset
        $this->seek($offset, self::SEEK_START);

        // Read data
        $data = $this->read($length);

        // Restore original position
        $this->seek($currentPos, self::SEEK_START);

        return $data;
    }

    public function writeAt(int $offset, string $data): int
    {
        // Save current position
        $currentPos = $this->seek(0, self::SEEK_CURRENT);

        // Seek to target offset
        $this->seek($offset, self::SEEK_START);

        // Write data
        $n = $this->write($data);

        // Restore original position
        $this->seek($currentPos, self::SEEK_START);

        return $n;
    }

    public function readFrom(Reader $reader): int
    {
        $totalWritten = 0;
        $bufferSize = 8192;

        while (true) {
            $data = $reader->read($bufferSize);
            if ($data === null || $data === '') {
                break;
            }

            $n = $this->write($data);
            $totalWritten += $n;

            if ($n < strlen($data)) {
                break;
            }
        }

        return $totalWritten;
    }

    public function writeTo(Writer $writer): int
    {
        $totalWritten = 0;
        $bufferSize = 8192;

        while (true) {
            $data = $this->read($bufferSize);
            if ($data === null || $data === '') {
                break;
            }

            $n = $writer->write($data);
            $totalWritten += $n;

            if ($n < strlen($data)) {
                break;
            }
        }

        return $totalWritten;
    }

    /**
     * Cast underlying kernel handle into a Kernel IO wrapper by bitflags.
     *
     * @return mixed Kernel IO object (AsyncReader/AsyncWriter/AsyncReadWriter/...)
     */
    public function castTo(int $type)
    {
        return $this->inner->castTo($type);
    }
}
