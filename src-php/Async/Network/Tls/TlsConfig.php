<?php

namespace Async\Network\Tls;

use Async\Kernel\Network\TlsConfig as KernelTlsConfig;

/**
 * TLS configuration builder
 */
class TlsConfig
{
    private KernelTlsConfig $inner;

    private function __construct(KernelTlsConfig $inner)
    {
        $this->inner = $inner;
    }

    /**
     * Create default TLS config with webpki root certificates
     */
    public static function default(): self
    {
        $config = KernelTlsConfig::default();
        if (!$config) {
            throw new \RuntimeException("Failed to create default TLS config");
        }
        return new self($config);
    }

    /**
     * Create TLS config without certificate verification (DANGEROUS - only for testing)
     */
    public static function dangerousNoVerify(): self
    {
        $config = KernelTlsConfig::dangerous_no_verify();
        if (!$config) {
            throw new \RuntimeException("Failed to create no-verify TLS config");
        }
        return new self($config);
    }

    /**
     * Create TLS config with custom CA certificate file
     */
    public static function withCaFile(string $caFile): self
    {
        $config = KernelTlsConfig::with_ca_file($caFile);
        if (!$config) {
            throw new \RuntimeException("Failed to create TLS config with CA file: $caFile");
        }
        return new self($config);
    }

    /**
     * Create TLS config with custom CA certificate from PEM string
     */
    public static function withCaString(string $caPem): self
    {
        $config = KernelTlsConfig::with_ca_string($caPem);
        if (!$config) {
            throw new \RuntimeException("Failed to create TLS config with CA string");
        }
        return new self($config);
    }

    /**
     * Create TLS config with client certificate authentication (mTLS)
     * @param string|null $caFile Optional CA file (null to use webpki roots)
     * @param string $certFile Client certificate file
     * @param string $keyFile Client private key file
     */
    public static function withClientCert(?string $caFile, string $certFile, string $keyFile): self
    {
        $config = KernelTlsConfig::with_client_cert($caFile, $certFile, $keyFile);
        if (!$config) {
            throw new \RuntimeException("Failed to create TLS config with client certificate");
        }
        return new self($config);
    }

    /**
     * Create TLS config with client certificate authentication from PEM strings
     * @param string|null $caPem Optional CA PEM (null to use webpki roots)
     * @param string $certPem Client certificate PEM
     * @param string $keyPem Client private key PEM
     */
    public static function withClientCertString(?string $caPem, string $certPem, string $keyPem): self
    {
        $config = KernelTlsConfig::with_client_cert_string($caPem, $certPem, $keyPem);
        if (!$config) {
            throw new \RuntimeException("Failed to create TLS config with client certificate string");
        }
        return new self($config);
    }

    /**
     * Get the underlying kernel TLS config
     * @internal
     */
    public function unwrap(): KernelTlsConfig
    {
        return $this->inner;
    }
}
