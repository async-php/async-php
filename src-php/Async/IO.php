<?php

namespace Async;

use Async\IO\Wrapper\ReaderWrapper;
use Async\IO\Wrapper\WriterWrapper;
use Async\IO\Wrapper\SeekerWrapper;
use Async\IO\Wrapper\BufReaderWrapper;
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
use Async\Kernel\IO\AsyncReader;
use Async\Kernel\IO\AsyncWriter;
use Async\Kernel\IO\AsyncSeeker;
use Async\Kernel\IO\AsyncBufReader;

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
        $channel = self::spawnIO($reader)->unwrap();
        return new \Async\Kernel\IO\PhpReader($channel);
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
        $channel = self::spawnIO($writer)->unwrap();
        return new \Async\Kernel\IO\PhpWriter($channel);
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
        $channel = self::spawnIO($seeker)->unwrap();
        return new \Async\Kernel\IO\PhpSeeker($channel);
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
        $channel = self::spawnIO($reader)->unwrap();
        return new \Async\Kernel\IO\PhpBufReader($channel);
    }

    /**
     * Wraps an IO object into a Channel for asynchronous operations in coroutines
     *
     * This method creates a new coroutine to handle IO operations and communicates through a Channel.
     * It enables non-blocking IO operations by delegating work to a separate coroutine context.
     *
     * The spawned fiber will terminate when:
     * - Channel is closed (pop returns null)
     * - Receives '__close__' command
     *
     * @param object $io The IO object to be wrapped
     * @return Channel Returns a Channel for communicating with the IO object
     */
    public static function spawnIO($io): Channel
    {
        $channel = new Channel();
        Kernel::spawn(function () use ($io, $channel) {
            while (true) {
                var_dump("spawnIO");
                [$request, $ok] = $channel->pop();
                var_dump($request, $ok);
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

                [, $ok] = $channel->push($result);
                if (!$ok) {
                    break;
                }
            }
        });
        return $channel;
    }
}
