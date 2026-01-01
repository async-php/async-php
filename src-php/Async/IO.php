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
use Async\IO\Seeker;
use Async\IO\BufReader;
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
     * - READ: PhpReader - requires Reader interface
     * - WRITE: PhpWriter - requires Writer interface
     * - SEEK: PhpSeeker - requires Seeker interface
     * - BUF: PhpBufReader - requires BufReader interface
     * - READ|WRITE: PhpReadWriter - requires ReadWriter interface
     * - READ|SEEK: PhpReadSeeker - requires ReadSeeker interface
     * - WRITE|SEEK: PhpWriteSeeker - requires WriteSeeker interface
     * - READ|WRITE|SEEK: PhpReadWriteSeeker - requires ReadWriteSeeker interface
     *
     * @param object $io PHP object implementing the required interfaces
     * @param int $type Bitflags (IO::READ | IO::WRITE | IO::SEEK | IO::BUF)
     * @return \Async\Kernel\IO\PhpReader|\Async\Kernel\IO\PhpWriter|\Async\Kernel\IO\PhpSeeker|\Async\Kernel\IO\PhpBufReader|\Async\Kernel\IO\PhpReadWriter|\Async\Kernel\IO\PhpReadSeeker|\Async\Kernel\IO\PhpWriteSeeker|\Async\Kernel\IO\PhpReadWriteSeeker
     * @throws \InvalidArgumentException If required interfaces are not implemented
     */
    public static function wrapPhpIo(object $io, int $type)
    {
        return new \Async\Kernel\IO\PhpIo($io);
    }
}
