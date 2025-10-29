<?php

// Sanity check for script logic
interface A {}
interface B extends A {}
$ref = new ReflectionClass('B');
if (empty($ref->getInterfaceNames())) {
    echo "ERROR: Script logic for interface inheritance check is BROKEN.\n";
} else {
    // echo "Script logic check passed.\n";
}

$interfaces = [
    '\Async\Kernel\IO\Reader',
    '\Async\Kernel\IO\Writer',
    '\Async\Kernel\IO\Closer',
    '\Async\Kernel\IO\ReadCloser',
    '\Async\Kernel\IO\WriteCloser',
    '\Async\Kernel\IO\ReaderAt',
    '\Async\Kernel\IO\WriterAt',
    '\Async\Kernel\IO\Seeker',
    '\Async\Kernel\IO\ReadSeeker',
    '\Async\Kernel\IO\WriteSeeker',
    '\Async\Kernel\IO\ReadWriter',
    '\Async\Kernel\IO\ReadWriteSeeker',
    '\Async\Kernel\IO\ReaderFrom',
    '\Async\Kernel\IO\WriterTo',
    '\Async\Kernel\IO\ByteReader',
    '\Async\Kernel\IO\ByteScanner',
    '\Async\Kernel\IO\StringReader',
];

foreach ($interfaces as $interfaceName) {
    echo "Interface: $interfaceName\n";
    if (!interface_exists($interfaceName)) {
        echo "  [NOT FOUND]\n";
        continue;
    }

    $ref = new ReflectionClass($interfaceName);
    
    // Check parent interfaces
    $parentInterfaces = $ref->getInterfaceNames();
    if (!empty($parentInterfaces)) {
        echo "  Extends: " . implode(', ', $parentInterfaces) . "\n";
    }

    $methods = $ref->getMethods();
    if (empty($methods)) {
        echo "  Methods: (none)\n";
    } else {
        echo "  Methods:\n";
        foreach ($methods as $method) {
            $params = [];
            foreach ($method->getParameters() as $param) {
                $pStr = '';
                if ($param->hasType()) {
                    $type = $param->getType();
                    if ($type instanceof ReflectionUnionType) {
                        $pStr .= implode('|', $type->getTypes()) . ' ';
                    } else {
                         $pStr .= $type->getName() . ' ';
                    }
                }
                $pStr .= '$' . $param->getName();
                $params[] = $pStr;
            }
            
            $retStr = '';
            if ($method->hasReturnType()) {
                $returnType = $method->getReturnType();
                if ($returnType instanceof ReflectionUnionType) {
                     $retStr = ': ' . implode('|', $returnType->getTypes());
                } else {
                     $retStr = ': ' . $returnType->getName();
                }
            }
            
            echo "    - " . $method->getName() . "(" . implode(', ', $params) . ")" . $retStr . "\n";
        }
    }
    echo str_repeat('-', 40) . "\n";
}