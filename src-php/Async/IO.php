<?php

namespace Async;

use Async\IO\Wrapper\ReaderWrapper;
use Async\IO\Wrapper\WriterWrapper;
use Async\IO\Wrapper\SeekerWrapper;
use Async\IO\Wrapper\BufReaderWrapper;
use Async\IO\Wrapper\ReadWriterWrapper;
use Async\IO\Wrapper\ReadSeekerWrapper;
use Async\IO\Wrapper\WriteSeekerWrapper;
use Async\IO\Wrapper\ReadWriteSeekerWrapper;
use Async\IO\Reader;
use Async\IO\Writer;
use Async\Kernel\IO\AsyncReader;
use Async\Kernel\IO\AsyncWriter;
use Async\Kernel\IO\AsyncSeeker;
use Async\Kernel\IO\AsyncBufReader;
use Async\Kernel\IO\AsyncReadWriter;
use Async\Kernel\IO\AsyncReadSeeker;
use Async\Kernel\IO\AsyncWriteSeeker;
use Async\Kernel\IO\AsyncReadWriteSeeker;

/**
 * IO provides type conversion methods between Kernel IO types and Wrapper types
 */
class IO
{
    /**
     * Kernel IO cast bitflags (re-exported for userland).
     *
     * Values are defined by the extension as constants:
     * - ASYNC_READ
     * - ASYNC_WRITE
     * - ASYNC_SEEK
     * - ASYNC_BUF
     */
    public const READ = ASYNC_READ;
    public const WRITE = ASYNC_WRITE;
    public const SEEK = ASYNC_SEEK;
    public const BUF = ASYNC_BUF;

    /**
     * Convert kernel IO object to wrapper object based on its type
     *
     * @param object $kernelIo Kernel IO object (AsyncReader, AsyncWriter, etc.)
     * @return ReaderWrapper|WriterWrapper|SeekerWrapper|BufReaderWrapper|ReadWriterWrapper|ReadSeekerWrapper|WriteSeekerWrapper|ReadWriteSeekerWrapper
     */
    public static function kernelToWrapper(object $kernelIo)
    {
        return match (true) {
            $kernelIo instanceof AsyncReadWriteSeeker => new ReadWriteSeekerWrapper($kernelIo),
            $kernelIo instanceof AsyncReadSeeker => new ReadSeekerWrapper($kernelIo),
            $kernelIo instanceof AsyncWriteSeeker => new WriteSeekerWrapper($kernelIo),
            $kernelIo instanceof AsyncReadWriter => new ReadWriterWrapper($kernelIo),
            $kernelIo instanceof AsyncBufReader => new BufReaderWrapper($kernelIo),
            $kernelIo instanceof AsyncReader => new ReaderWrapper($kernelIo),
            $kernelIo instanceof AsyncWriter => new WriterWrapper($kernelIo),
            $kernelIo instanceof AsyncSeeker => new SeekerWrapper($kernelIo),
            default => throw new \InvalidArgumentException('Unknown kernel IO type: ' . get_class($kernelIo)),
        };
    }

    /**
     * Copy data from a Reader to a Writer
     *
     * @param Reader $reader Source reader
     * @param Writer $writer Destination writer
     * @param int $bufferSize Buffer size for copying (default 8192)
     * @return int Total bytes copied
     */
    public static function copy(Reader $reader, Writer $writer, int $bufferSize = 8192): int
    {
        $totalWritten = 0;
        while (true) {
            $data = $reader->read($bufferSize);
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
     * Wrap a PHP IO object into a PhpIO kernel type (for tokio async usage)
     *
     * Based on the bitflags provided, creates the appropriate PhpIO type:
     * - READ: PhpReader - requires read($length) method
     * - WRITE: PhpWriter - requires write($data) and flush() methods
     * - SEEK: PhpSeeker - requires seek($offset, $whence) method
     * - BUF: PhpBufReader - requires read_line() method
     * - READ|WRITE: PhpReadWriter
     * - READ|SEEK: PhpReadSeeker
     * - WRITE|SEEK: PhpWriteSeeker
     * - READ|WRITE|SEEK: PhpReadWriteSeeker
     *
     * @param object $io PHP object implementing the required methods
     * @param int $type Bitflags (IO::READ | IO::WRITE | IO::SEEK | IO::BUF)
     * @return \Async\Kernel\IO\PhpReader|\Async\Kernel\IO\PhpWriter|\Async\Kernel\IO\PhpSeeker|\Async\Kernel\IO\PhpBufReader|\Async\Kernel\IO\PhpReadWriter|\Async\Kernel\IO\PhpReadSeeker|\Async\Kernel\IO\PhpWriteSeeker|\Async\Kernel\IO\PhpReadWriteSeeker
     */
    public static function wrapPhpIo(object $io, int $type)
    {
        [$requestChannel, $responseChannel] = self::spawnIO($io);
        $reqChan = $requestChannel->unwrap();
        $resChan = $responseChannel->unwrap();

        // Check for combined types first (most specific to least specific)
        if (($type & self::READ) && ($type & self::WRITE) && ($type & self::SEEK)) {
            return new \Async\Kernel\IO\PhpReadWriteSeeker($reqChan, $resChan);
        }
        if (($type & self::READ) && ($type & self::SEEK)) {
            return new \Async\Kernel\IO\PhpReadSeeker($reqChan, $resChan);
        }
        if (($type & self::WRITE) && ($type & self::SEEK)) {
            return new \Async\Kernel\IO\PhpWriteSeeker($reqChan, $resChan);
        }
        if (($type & self::READ) && ($type & self::WRITE)) {
            return new \Async\Kernel\IO\PhpReadWriter($reqChan, $resChan);
        }

        // Check for single types
        if ($type & self::BUF) {
            return new \Async\Kernel\IO\PhpBufReader($reqChan, $resChan);
        }
        if ($type & self::READ) {
            return new \Async\Kernel\IO\PhpReader($reqChan, $resChan);
        }
        if ($type & self::WRITE) {
            return new \Async\Kernel\IO\PhpWriter($reqChan, $resChan);
        }
        if ($type & self::SEEK) {
            return new \Async\Kernel\IO\PhpSeeker($reqChan, $resChan);
        }

        throw new \InvalidArgumentException('Invalid IO type flags: ' . $type);
    }

    /**
     * Wraps an IO object into dual Channels for asynchronous operations in coroutines
     *
     * This method creates a new coroutine to handle IO operations and communicates through two Channels:
     * - Request channel: for sending method calls to the spawned fiber
     * - Response channel: for receiving results from the spawned fiber
     *
     * It enables non-blocking IO operations by delegating work to a separate coroutine context.
     *
     * The spawned fiber will terminate when:
     * - Request channel is closed (pop returns null)
     * - Receives '__close__' command
     *
     * @param object $io The IO object to be wrapped
     * @return array Returns [requestChannel, responseChannel]
     */
    private static function spawnIO($io): array
    {
        $requestChannel = new Channel();
        $responseChannel = new Channel();

        Kernel::spawn(function () use ($io, $requestChannel, $responseChannel) {
            while (true) {
                [$request, $ok] = $requestChannel->pop();
                if (!$ok) {
                    break;
                }

                if (!is_array($request) || count($request) !== 2) {
                    throw new \InvalidArgumentException('Request must be [$method, $args] tuple');
                }

                [$method, $args] = $request;

                // Terminate on close command
                if ($method === '__close__') {
                    break;
                }

                $result = $io->$method(...$args);

                $ok = $responseChannel->push($result);
                if (!$ok) {
                    break;
                }
            }
        });

        return [$requestChannel, $responseChannel];
    }
}
