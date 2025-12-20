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
use Async\IO\Adapter\ByteReaderAdapter;
use Async\IO\Adapter\ByteWriterAdapter;
use Async\IO\Adapter\StringReaderAdapter;
use Async\IO\Adapter\StringWriterAdapter;
use Async\IO\Adapter\RuneReaderAdapter;
use Async\IO\Adapter\ReaderAtAdapter;
use Async\IO\Adapter\WriterAtAdapter;
use Async\IO\Adapter\ReaderFromAdapter;
use Async\IO\Adapter\WriterToAdapter;
use Async\IO\Reader;
use Async\IO\Writer;
use Async\IO\Seeker;
use Async\IO\ReadWriter;
use Async\IO\ReadSeeker;
use Async\IO\WriteSeeker;
use Async\IO\ReadWriteSeeker;
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
        return self::asWriterTo($reader, $bufferSize)->writeTo($writer);
    }

    /**
     * Wrap a PHP reader object into PhpReader (for tokio async usage)
     *
     * The object should have a read($length) method that returns string|null
     *
     * @param object $reader PHP object with read($length) method
     * @return \Async\Kernel\IO\PhpReader
     */
    public static function wrapPhpReader($reader): \Async\Kernel\IO\PhpReader
    {
        [$requestChannel, $responseChannel] = self::spawnIO($reader);
        return new \Async\Kernel\IO\PhpReader($requestChannel->unwrap(), $responseChannel->unwrap());
    }

    /**
     * Wrap a PHP writer object into PhpWriter (for tokio async usage)
     *
     * The object should have write($data) and flush() methods
     *
     * @param object $writer PHP object with write($data) and flush() methods
     * @return \Async\Kernel\IO\PhpWriter
     */
    public static function wrapPhpWriter($writer): \Async\Kernel\IO\PhpWriter
    {
        [$requestChannel, $responseChannel] = self::spawnIO($writer);
        return new \Async\Kernel\IO\PhpWriter($requestChannel->unwrap(), $responseChannel->unwrap());
    }

    /**
     * Wrap a PHP seeker object into PhpSeeker (for tokio async usage)
     *
     * The object should have a seek($offset, $whence) method
     *
     * @param object $seeker PHP object with seek($offset, $whence) method
     * @return \Async\Kernel\IO\PhpSeeker
     */
    public static function wrapPhpSeeker($seeker): \Async\Kernel\IO\PhpSeeker
    {
        [$requestChannel, $responseChannel] = self::spawnIO($seeker);
        return new \Async\Kernel\IO\PhpSeeker($requestChannel->unwrap(), $responseChannel->unwrap());
    }

    /**
     * Wrap a PHP buffered reader object into PhpBufReader (for tokio async usage)
     *
     * The object should have read_line() and read($length) methods
     *
     * @param object $reader PHP object with read_line() and read($length) methods
     * @return \Async\Kernel\IO\PhpBufReader
     */
    public static function wrapPhpBufReader($reader): \Async\Kernel\IO\PhpBufReader
    {
        [$requestChannel, $responseChannel] = self::spawnIO($reader);
        return new \Async\Kernel\IO\PhpBufReader($requestChannel->unwrap(), $responseChannel->unwrap());
    }

    /**
     * Wrap a PHP read-writer object into PhpReadWriter (for tokio async usage)
     *
     * The object should have read($length), write($data), and flush() methods
     *
     * @param object $readWriter PHP object implementing read, write, and flush methods
     * @return \Async\Kernel\IO\PhpReadWriter
     */
    public static function wrapPhpReadWriter($readWriter): \Async\Kernel\IO\PhpReadWriter
    {
        [$requestChannel, $responseChannel] = self::spawnIO($readWriter);
        return new \Async\Kernel\IO\PhpReadWriter($requestChannel->unwrap(), $responseChannel->unwrap());
    }

    /**
     * Wrap a PHP read-seeker object into PhpReadSeeker (for tokio async usage)
     *
     * The object should have read($length) and seek($offset, $whence) methods
     *
     * @param object $readSeeker PHP object implementing read and seek methods
     * @return \Async\Kernel\IO\PhpReadSeeker
     */
    public static function wrapPhpReadSeeker($readSeeker): \Async\Kernel\IO\PhpReadSeeker
    {
        [$requestChannel, $responseChannel] = self::spawnIO($readSeeker);
        return new \Async\Kernel\IO\PhpReadSeeker($requestChannel->unwrap(), $responseChannel->unwrap());
    }

    /**
     * Wrap a PHP write-seeker object into PhpWriteSeeker (for tokio async usage)
     *
     * The object should have write($data), flush(), and seek($offset, $whence) methods
     *
     * @param object $writeSeeker PHP object implementing write, flush, and seek methods
     * @return \Async\Kernel\IO\PhpWriteSeeker
     */
    public static function wrapPhpWriteSeeker($writeSeeker): \Async\Kernel\IO\PhpWriteSeeker
    {
        [$requestChannel, $responseChannel] = self::spawnIO($writeSeeker);
        return new \Async\Kernel\IO\PhpWriteSeeker($requestChannel->unwrap(), $responseChannel->unwrap());
    }

    /**
     * Wrap a PHP read-write-seeker object into PhpReadWriteSeeker (for tokio async usage)
     *
     * The object should have read($length), write($data), flush(), and seek($offset, $whence) methods
     *
     * @param object $readWriteSeeker PHP object implementing read, write, flush, and seek methods
     * @return \Async\Kernel\IO\PhpReadWriteSeeker
     */
    public static function wrapPhpReadWriteSeeker($readWriteSeeker): \Async\Kernel\IO\PhpReadWriteSeeker
    {
        [$requestChannel, $responseChannel] = self::spawnIO($readWriteSeeker);
        return new \Async\Kernel\IO\PhpReadWriteSeeker($requestChannel->unwrap(), $responseChannel->unwrap());
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
