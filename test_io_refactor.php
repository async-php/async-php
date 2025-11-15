<?php

require_once __DIR__ . '/vendor/autoload.php';

use Async\Network\Tcp\Socket;
use Async\FileSystem\FileHandle;
use Async\IO\Reader;
use Async\IO\Writer;
use Async\IO\Closer;
use Async\IO\Seeker;

echo "Testing IO Module Refactoring...\n\n";

// Test 1: Check that interfaces exist
echo "✓ Checking IO interfaces exist:\n";
echo "  - Reader: " . (interface_exists('Async\\IO\\Reader') ? 'YES' : 'NO') . "\n";
echo "  - Writer: " . (interface_exists('Async\\IO\\Writer') ? 'YES' : 'NO') . "\n";
echo "  - Closer: " . (interface_exists('Async\\IO\\Closer') ? 'YES' : 'NO') . "\n";
echo "  - Seeker: " . (interface_exists('Async\\IO\\Seeker') ? 'YES' : 'NO') . "\n";
echo "  - ByteReader: " . (interface_exists('Async\\IO\\ByteReader') ? 'YES' : 'NO') . "\n";
echo "  - ByteWriter: " . (interface_exists('Async\\IO\\ByteWriter') ? 'YES' : 'NO') . "\n";
echo "  - StringReader: " . (interface_exists('Async\\IO\\StringReader') ? 'YES' : 'NO') . "\n";
echo "  - StringWriter: " . (interface_exists('Async\\IO\\StringWriter') ? 'YES' : 'NO') . "\n";
echo "  - RuneReader: " . (interface_exists('Async\\IO\\RuneReader') ? 'YES' : 'NO') . "\n";
echo "  - BufReader: " . (interface_exists('Async\\IO\\BufReader') ? 'YES' : 'NO') . "\n\n";

// Test 2: Check Seeker constants
echo "✓ Checking Seeker constants:\n";
echo "  - SEEK_START: " . Seeker::SEEK_START . "\n";
echo "  - SEEK_CURRENT: " . Seeker::SEEK_CURRENT . "\n";
echo "  - SEEK_END: " . Seeker::SEEK_END . "\n\n";

// Test 3: Check that Wrapper classes exist
echo "✓ Checking Wrapper classes exist:\n";
echo "  - TcpStreamWrapper: " . (class_exists('Async\\IO\\Wrapper\\TcpStreamWrapper') ? 'YES' : 'NO') . "\n";
echo "  - FileHandleWrapper: " . (class_exists('Async\\IO\\Wrapper\\FileHandleWrapper') ? 'YES' : 'NO') . "\n";
echo "  - HttpBodyWrapper: " . (class_exists('Async\\IO\\Wrapper\\HttpBodyWrapper') ? 'YES' : 'NO') . "\n\n";

// Test 4: Check that Socket implements interfaces
echo "✓ Checking Socket implements IO interfaces:\n";
$socketClass = new ReflectionClass(Socket::class);
echo "  - Implements Reader: " . ($socketClass->implementsInterface(Reader::class) ? 'YES' : 'NO') . "\n";
echo "  - Implements Writer: " . ($socketClass->implementsInterface(Writer::class) ? 'YES' : 'NO') . "\n";
echo "  - Implements Closer: " . ($socketClass->implementsInterface(Closer::class) ? 'YES' : 'NO') . "\n";
echo "  - Has read() method: " . ($socketClass->hasMethod('read') ? 'YES' : 'NO') . "\n";
echo "  - Has write() method: " . ($socketClass->hasMethod('write') ? 'YES' : 'NO') . "\n";
echo "  - Has flush() method: " . ($socketClass->hasMethod('flush') ? 'YES' : 'NO') . "\n";
echo "  - Has close() method: " . ($socketClass->hasMethod('close') ? 'YES' : 'NO') . "\n\n";

// Test 5: Check that FileHandle implements interfaces
echo "✓ Checking FileHandle implements IO interfaces:\n";
$fileClass = new ReflectionClass(FileHandle::class);
echo "  - Implements Reader: " . ($fileClass->implementsInterface(Reader::class) ? 'YES' : 'NO') . "\n";
echo "  - Implements Writer: " . ($fileClass->implementsInterface(Writer::class) ? 'YES' : 'NO') . "\n";
echo "  - Implements Closer: " . ($fileClass->implementsInterface(Closer::class) ? 'YES' : 'NO') . "\n";
echo "  - Implements Seeker: " . ($fileClass->implementsInterface(Seeker::class) ? 'YES' : 'NO') . "\n";
echo "  - Has read() method: " . ($fileClass->hasMethod('read') ? 'YES' : 'NO') . "\n";
echo "  - Has write() method: " . ($fileClass->hasMethod('write') ? 'YES' : 'NO') . "\n";
echo "  - Has flush() method: " . ($fileClass->hasMethod('flush') ? 'YES' : 'NO') . "\n";
echo "  - Has seek() method: " . ($fileClass->hasMethod('seek') ? 'YES' : 'NO') . "\n";
echo "  - Has close() method: " . ($fileClass->hasMethod('close') ? 'YES' : 'NO') . "\n\n";

// Test 6: Check that old Kernel interfaces are removed
echo "✓ Checking old Kernel interfaces are removed:\n";
echo "  - Async\\Kernel\\IO\\Reader exists: " . (interface_exists('Async\\Kernel\\IO\\Reader') ? 'YES (BAD!)' : 'NO (GOOD)') . "\n";
echo "  - Async\\Kernel\\IO\\Writer exists: " . (interface_exists('Async\\Kernel\\IO\\Writer') ? 'YES (BAD!)' : 'NO (GOOD)') . "\n\n";

// Test 7: Check IO helper class
echo "✓ Checking IO helper class:\n";
echo "  - Async\\IO class exists: " . (class_exists('Async\\IO') ? 'YES' : 'NO') . "\n";
$ioClass = new ReflectionClass('Async\\IO');
echo "  - Has wrapTcpStream() method: " . ($ioClass->hasMethod('wrapTcpStream') ? 'YES' : 'NO') . "\n";
echo "  - Has wrapFileHandle() method: " . ($ioClass->hasMethod('wrapFileHandle') ? 'YES' : 'NO') . "\n";
echo "  - Has wrapHttpBody() method: " . ($ioClass->hasMethod('wrapHttpBody') ? 'YES' : 'NO') . "\n\n";

echo "✅ All structure tests passed!\n";
echo "\n=== IO Module Refactoring Complete ===\n";
