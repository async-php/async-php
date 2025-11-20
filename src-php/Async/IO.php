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
     * Wrap AsyncReader into a ReaderWrapper
     *
     * @param AsyncReader $reader The kernel async reader
     * @return ReaderWrapper Reader interface implementation
     */
    public static function wrapReader(AsyncReader $reader): ReaderWrapper
    {
        return new ReaderWrapper($reader);
    }

    /**
     * Wrap AsyncWriter into a WriterWrapper
     *
     * @param AsyncWriter $writer The kernel async writer
     * @return WriterWrapper Writer interface implementation
     */
    public static function wrapWriter(AsyncWriter $writer): WriterWrapper
    {
        return new WriterWrapper($writer);
    }

    /**
     * Wrap AsyncSeeker into a SeekerWrapper
     *
     * @param AsyncSeeker $seeker The kernel async seeker
     * @return SeekerWrapper Seeker interface implementation
     */
    public static function wrapSeeker(AsyncSeeker $seeker): SeekerWrapper
    {
        return new SeekerWrapper($seeker);
    }

    /**
     * Wrap AsyncBufReader into a BufReaderWrapper
     *
     * @param AsyncBufReader $reader The kernel async buffered reader
     * @return BufReaderWrapper BufReader interface implementation
     */
    public static function wrapBufReader(AsyncBufReader $reader): BufReaderWrapper
    {
        return new BufReaderWrapper($reader);
    }

    /**
     * Wrap AsyncReadWriter into a ReadWriterWrapper
     *
     * @param AsyncReadWriter $readWriter The kernel async read-writer
     * @return ReadWriterWrapper ReadWriter interface implementation
     */
    public static function wrapReadWriter(AsyncReadWriter $readWriter): ReadWriterWrapper
    {
        return new ReadWriterWrapper($readWriter);
    }

    /**
     * Wrap AsyncReadSeeker into a ReadSeekerWrapper
     *
     * @param AsyncReadSeeker $readSeeker The kernel async read-seeker
     * @return ReadSeekerWrapper ReadSeeker interface implementation
     */
    public static function wrapReadSeeker(AsyncReadSeeker $readSeeker): ReadSeekerWrapper
    {
        return new ReadSeekerWrapper($readSeeker);
    }

    /**
     * Wrap AsyncWriteSeeker into a WriteSeekerWrapper
     *
     * @param AsyncWriteSeeker $writeSeeker The kernel async write-seeker
     * @return WriteSeekerWrapper WriteSeeker interface implementation
     */
    public static function wrapWriteSeeker(AsyncWriteSeeker $writeSeeker): WriteSeekerWrapper
    {
        return new WriteSeekerWrapper($writeSeeker);
    }

    /**
     * Wrap AsyncReadWriteSeeker into a ReadWriteSeekerWrapper
     *
     * @param AsyncReadWriteSeeker $readWriteSeeker The kernel async read-write-seeker
     * @return ReadWriteSeekerWrapper ReadWriteSeeker interface implementation
     */
    public static function wrapReadWriteSeeker(AsyncReadWriteSeeker $readWriteSeeker): ReadWriteSeekerWrapper
    {
        return new ReadWriteSeekerWrapper($readWriteSeeker);
    }

    /**
     * Create a ByteReader adapter from a Reader
     *
     * @param Reader $reader The reader to adapt
     * @return ByteReaderAdapter ByteScanner implementation
     */
    public static function asByteReader(Reader $reader): ByteReaderAdapter
    {
        return new ByteReaderAdapter($reader);
    }

    /**
     * Create a ByteWriter adapter from a Writer
     *
     * @param Writer $writer The writer to adapt
     * @return ByteWriterAdapter ByteWriter implementation
     */
    public static function asByteWriter(Writer $writer): ByteWriterAdapter
    {
        return new ByteWriterAdapter($writer);
    }

    /**
     * Create a StringReader adapter from a Reader
     *
     * @param Reader $reader The reader to adapt
     * @return StringReaderAdapter StringReader implementation
     */
    public static function asStringReader(Reader $reader): StringReaderAdapter
    {
        return new StringReaderAdapter($reader);
    }

    /**
     * Create a StringWriter adapter from a Writer
     *
     * @param Writer $writer The writer to adapt
     * @return StringWriterAdapter StringWriter implementation
     */
    public static function asStringWriter(Writer $writer): StringWriterAdapter
    {
        return new StringWriterAdapter($writer);
    }

    /**
     * Create a RuneReader adapter from a Reader
     *
     * @param Reader $reader The reader to adapt
     * @return RuneReaderAdapter RuneScanner implementation
     */
    public static function asRuneReader(Reader $reader): RuneReaderAdapter
    {
        return new RuneReaderAdapter($reader);
    }

    /**
     * Create a ReaderAt adapter from a Reader and Seeker
     *
     * @param Reader $reader The reader to adapt
     * @param Seeker $seeker The seeker for positioning
     * @return ReaderAtAdapter ReaderAt implementation
     */
    public static function asReaderAt(Reader $reader, Seeker $seeker): ReaderAtAdapter
    {
        return new ReaderAtAdapter($reader, $seeker);
    }

    /**
     * Create a WriterAt adapter from a Writer and Seeker
     *
     * @param Writer $writer The writer to adapt
     * @param Seeker $seeker The seeker for positioning
     * @return WriterAtAdapter WriterAt implementation
     */
    public static function asWriterAt(Writer $writer, Seeker $seeker): WriterAtAdapter
    {
        return new WriterAtAdapter($writer, $seeker);
    }

    /**
     * Create a ReaderFrom adapter from a Writer
     *
     * @param Writer $writer The writer to copy data into
     * @param int $bufferSize Buffer size for copying (default 8192)
     * @return ReaderFromAdapter ReaderFrom implementation
     */
    public static function asReaderFrom(Writer $writer, int $bufferSize = 8192): ReaderFromAdapter
    {
        return new ReaderFromAdapter($writer, $bufferSize);
    }

    /**
     * Create a WriterTo adapter from a Reader
     *
     * @param Reader $reader The reader to copy data from
     * @param int $bufferSize Buffer size for copying (default 8192)
     * @return WriterToAdapter WriterTo implementation
     */
    public static function asWriterTo(Reader $reader, int $bufferSize = 8192): WriterToAdapter
    {
        return new WriterToAdapter($reader, $bufferSize);
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
