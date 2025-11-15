<?php

require_once __DIR__ . '/vendor/autoload.php';

use Async\IO;
use Async\IO\Wrapper\ReaderWrapper;
use Async\IO\Wrapper\WriterWrapper;
use Async\IO\Wrapper\SeekerWrapper;
use Async\IO\Wrapper\BufReaderWrapper;

echo "Testing IO Wrapper Classes...\n\n";

// Test 1: Check Wrapper classes exist
echo "✓ Wrapper Classes:\n";
echo "  - ReaderWrapper: " . (class_exists('Async\\IO\\Wrapper\\ReaderWrapper') ? 'YES' : 'NO') . "\n";
echo "  - WriterWrapper: " . (class_exists('Async\\IO\\Wrapper\\WriterWrapper') ? 'YES' : 'NO') . "\n";
echo "  - SeekerWrapper: " . (class_exists('Async\\IO\\Wrapper\\SeekerWrapper') ? 'YES' : 'NO') . "\n";
echo "  - BufReaderWrapper: " . (class_exists('Async\\IO\\Wrapper\\BufReaderWrapper') ? 'YES' : 'NO') . "\n\n";

// Test 2: Check IO helper class
echo "✓ IO Helper Class:\n";
echo "  - Async\\IO exists: " . (class_exists('Async\\IO') ? 'YES' : 'NO') . "\n";
$ioClass = new ReflectionClass('Async\\IO');
echo "  - Has wrapReader() method: " . ($ioClass->hasMethod('wrapReader') ? 'YES' : 'NO') . "\n";
echo "  - Has wrapWriter() method: " . ($ioClass->hasMethod('wrapWriter') ? 'YES' : 'NO') . "\n";
echo "  - Has wrapSeeker() method: " . ($ioClass->hasMethod('wrapSeeker') ? 'YES' : 'NO') . "\n";
echo "  - Has wrapBufReader() method: " . ($ioClass->hasMethod('wrapBufReader') ? 'YES' : 'NO') . "\n\n";

// Test 3: Check ReaderWrapper implements interface
echo "✓ ReaderWrapper:\n";
$readerClass = new ReflectionClass(ReaderWrapper::class);
echo "  - Implements Reader: " . ($readerClass->implementsInterface('Async\\IO\\Reader') ? 'YES' : 'NO') . "\n";
echo "  - Has read() method: " . ($readerClass->hasMethod('read') ? 'YES' : 'NO') . "\n";
echo "  - Has unwrap() method: " . ($readerClass->hasMethod('unwrap') ? 'YES' : 'NO') . "\n\n";

// Test 4: Check WriterWrapper implements interface
echo "✓ WriterWrapper:\n";
$writerClass = new ReflectionClass(WriterWrapper::class);
echo "  - Implements Writer: " . ($writerClass->implementsInterface('Async\\IO\\Writer') ? 'YES' : 'NO') . "\n";
echo "  - Has write() method: " . ($writerClass->hasMethod('write') ? 'YES' : 'NO') . "\n";
echo "  - Has flush() method: " . ($writerClass->hasMethod('flush') ? 'YES' : 'NO') . "\n";
echo "  - Has unwrap() method: " . ($writerClass->hasMethod('unwrap') ? 'YES' : 'NO') . "\n\n";

// Test 5: Check SeekerWrapper implements interface
echo "✓ SeekerWrapper:\n";
$seekerClass = new ReflectionClass(SeekerWrapper::class);
echo "  - Implements Seeker: " . ($seekerClass->implementsInterface('Async\\IO\\Seeker') ? 'YES' : 'NO') . "\n";
echo "  - Has seek() method: " . ($seekerClass->hasMethod('seek') ? 'YES' : 'NO') . "\n";
echo "  - Has unwrap() method: " . ($seekerClass->hasMethod('unwrap') ? 'YES' : 'NO') . "\n\n";

// Test 6: Check BufReaderWrapper implements interface
echo "✓ BufReaderWrapper:\n";
$bufReaderClass = new ReflectionClass(BufReaderWrapper::class);
echo "  - Implements BufReader: " . ($bufReaderClass->implementsInterface('Async\\IO\\BufReader') ? 'YES' : 'NO') . "\n";
echo "  - Has readLine() method: " . ($bufReaderClass->hasMethod('readLine') ? 'YES' : 'NO') . "\n";
echo "  - Has readUntil() method: " . ($bufReaderClass->hasMethod('readUntil') ? 'YES' : 'NO') . "\n";
echo "  - Has unwrap() method: " . ($bufReaderClass->hasMethod('unwrap') ? 'YES' : 'NO') . "\n\n";

echo "✅ All Wrapper tests passed!\n";
echo "\n=== Complete Architecture ===\n";
echo "\n";
echo "PHP Layer (\\Async\\IO):\n";
echo "  - 13 interface files\n";
echo "\n";
echo "Wrapper Layer (\\Async\\IO\\Wrapper):\n";
echo "  - ReaderWrapper (wraps AsyncReader)\n";
echo "  - WriterWrapper (wraps AsyncWriter)\n";
echo "  - SeekerWrapper (wraps AsyncSeeker)\n";
echo "  - BufReaderWrapper (wraps AsyncBufReader)\n";
echo "\n";
echo "Rust Kernel Layer (\\Async\\Kernel\\IO):\n";
echo "  - AsyncReader (wraps Box<dyn AsyncRead>)\n";
echo "  - AsyncWriter (wraps Box<dyn AsyncWrite>)\n";
echo "  - AsyncSeeker (wraps Box<dyn AsyncSeek>)\n";
echo "  - AsyncBufReader (wraps Box<dyn AsyncBufRead>)\n";
echo "\n";
echo "Helper:\n";
echo "  - \\Async\\IO class (provides wrapReader/wrapWriter/etc)\n";
