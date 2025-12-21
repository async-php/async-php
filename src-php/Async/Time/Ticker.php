<?php

namespace Async\Time;

use Async\Kernel\Ticker as KernelTicker;
use Async\Kernel\Time as KernelTime;
use Fiber;
use IteratorAggregate;
use Traversable;

/**
 * Ticker provides a mechanism for executing code at regular intervals.
 *
 * Example:
 * ```php
 * $ticker = Ticker::create(1000); // Tick every 1 second
 * while ($ticker->isRunning()) {
 *     $ticker->tick();
 *     echo "Tick at " . date('H:i:s') . "\n";
 * }
 * ```
 */
class Ticker implements IteratorAggregate
{
    private KernelTicker $inner;

    /**
     * Create a new Ticker instance (use Ticker::create() instead)
     *
     * @param KernelTicker $inner The kernel ticker instance
     */
    public function __construct(KernelTicker $inner)
    {
        $this->inner = $inner;
    }

    /**
     * Create a new ticker that ticks at the specified interval
     *
     * @param int $intervalMs Interval in milliseconds between ticks
     * @return self
     */
    public static function create(int $intervalMs): self
    {
        $kernelTicker = KernelTime::createTicker($intervalMs);
        return new self($kernelTicker);
    }

    /**
     * Wait for the next tick
     *
     * This method will suspend the current fiber until the next tick occurs.
     *
     * @return void
     */
    public function tick(): void
    {
        $future = $this->inner->nextTick();
        Fiber::suspend($future);
    }

    /**
     * Stop the ticker permanently
     *
     * @return void
     */
    public function stop(): void
    {
        $this->inner->stop();
    }

    /**
     * Reset the ticker (restart from beginning with tick count = 0)
     *
     * @return void
     */
    public function reset(): void
    {
        $this->inner->reset();
    }

    /**
     * Get the interval in milliseconds
     *
     * @return int
     */
    public function getInterval(): int
    {
        return $this->inner->getInterval();
    }

    /**
     * Get the current tick count
     *
     * @return int
     */
    public function getTickCount(): int
    {
        return $this->inner->getTickCount();
    }

    /**
     * Set the maximum number of ticks (ticker will auto-stop after reaching this)
     *
     * @param int|null $max Maximum ticks (null or 0 means unlimited)
     * @return void
     */
    public function setMaxTicks(?int $max): void
    {
        $this->inner->setMaxTicks($max);
    }

    /**
     * Get the maximum number of ticks
     *
     * @return int|null
     */
    public function getMaxTicks(): ?int
    {
        return $this->inner->getMaxTicks();
    }

    /**
     * Check if the ticker is running
     *
     * @return bool
     */
    public function isRunning(): bool
    {
        return $this->inner->isRunning();
    }

    /**
     * Check if the ticker is stopped
     *
     * @return bool
     */
    public function isStopped(): bool
    {
        return $this->inner->isStopped();
    }

    /**
     * Get an iterator for the ticker
     *
     * This allows using the ticker in a foreach loop:
     * ```php
     * foreach ($ticker as $tick) {
     *     echo "Tick $tick\n";
     * }
     * ```
     *
     * @return Traversable
     */
    public function getIterator(): Traversable
    {
        while ($this->isRunning()) {
            $this->tick();
            yield $this->getTickCount();
        }
    }

    /**
     * Execute a callback on each tick
     *
     * @param callable $callback Function to call on each tick, receives tick count as parameter
     * @return void
     */
    public function forEach(callable $callback): void
    {
        while ($this->isRunning()) {
            $this->tick();
            $callback($this->getTickCount());
        }
    }
}
