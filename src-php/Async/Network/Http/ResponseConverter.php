<?php

namespace Async\Network\Http;

use Async\Kernel\Network\Http\HttpResponse as KernelResponse;
use Psr\Http\Message\ResponseInterface;

/**
 * Converts PSR-7 ResponseInterface to KernelResponse
 *
 * This is used to bridge PSR-15 middleware (which returns PSR-7 responses)
 * back to the kernel HTTP server (which expects KernelResponse).
 */
class ResponseConverter
{
    /**
     * Convert PSR-7 response to kernel response
     *
     * @param ResponseInterface $psr7Response
     * @return KernelResponse
     */
    public static function toKernelResponse(ResponseInterface $psr7Response): KernelResponse
    {
        $kernelResponse = new KernelResponse();

        // Set status code
        $kernelResponse->setStatus($psr7Response->getStatusCode());

        // Set headers
        foreach ($psr7Response->getHeaders() as $name => $values) {
            foreach ($values as $value) {
                $kernelResponse->setHeader($name, $value);
            }
        }

        // Set body
        $body = (string)$psr7Response->getBody();
        $kernelResponse->setBody($body);

        return $kernelResponse;
    }
}
