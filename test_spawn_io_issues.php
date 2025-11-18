<?php

require 'vendor/autoload.php';

use Async\Kernel;
use Async\IO;
use Async\Channel;

echo "=== Testing spawnIO Issues ===\n\n";

// Issue 1: Channel返回值检查错误
echo "Issue 1: Channel return value check\n";
class TestReader1 {
    public function read($length) {
        return "data";
    }
}

Kernel::run(function () {
    $reader = new TestReader1();
    $channel = IO::spawnIO($reader);

    // 关闭 channel
    $channel->close();

    // 检查 pop 返回什么
    $result = $channel->pop();
    echo "After close, pop returns: ";
    var_dump($result);

    // spawnIO 中使用 is_null($request) 检查，但这永远不会是 null
    // 正确的应该是检查 $result[1] === false
    echo "is_null(\$result): " . (is_null($result) ? 'true' : 'false') . "\n";
    echo "\$result[1] === false: " . ($result[1] === false ? 'true' : 'false') . "\n";
    echo "\n";
});

// Issue 2: 异常未捕获导致协程崩溃
echo "Issue 2: Uncaught exception in spawned fiber\n";
class TestReader2 {
    public function read($length) {
        throw new Exception("Read failed!");
    }
}

Kernel::run(function () {
    $reader = new TestReader2();
    $channel = IO::spawnIO($reader);

    // 推送一个读取请求
    echo "Pushing read request...\n";
    $channel->push(['read', [10]]);

    // spawn 的 fiber 会在这里崩溃，但外部不知道
    sleep(1);

    // 尝试获取结果 - 会永久阻塞
    echo "Waiting for result (will block)...\n";
    $result = $channel->pop(1.0); // 1秒超时
    echo "Result: ";
    var_dump($result);

    if ($result[1] === false) {
        echo "Failed to get result - fiber likely crashed\n";
    }
    echo "\n";
});

// Issue 3: 资源泄漏
echo "Issue 3: Resource leak on abnormal exit\n";
class TestResource {
    private $closed = false;

    public function read($length) {
        if ($this->closed) {
            throw new Exception("Resource already closed");
        }
        return "data";
    }

    public function close() {
        echo "[TestResource] Closing resource\n";
        $this->closed = true;
    }

    public function __destruct() {
        if (!$this->closed) {
            echo "[TestResource] WARNING: Resource not properly closed!\n";
        }
    }
}

Kernel::run(function () {
    $resource = new TestResource();
    $channel = IO::spawnIO($resource);

    // 直接关闭 channel，spawned fiber 会退出
    echo "Closing channel without calling resource->close()...\n";
    $channel->close();

    sleep(1);
    echo "Channel closed\n";
    // $resource 的 close() 方法从未被调用
    echo "\n";
});

echo "=== Test Completed ===\n";
