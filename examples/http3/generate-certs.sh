#!/bin/bash

# Generate self-signed certificates for HTTP/3 testing
# This script creates a certificate and private key for local testing only
# DO NOT use these certificates in production!

CERT_DIR="$(dirname "$0")/certs"
mkdir -p "$CERT_DIR"

echo "Generating self-signed certificate for HTTP/3 testing..."
echo "========================================================"

# Generate private key
openssl genrsa -out "$CERT_DIR/server.key" 2048

# Generate certificate signing request
openssl req -new -key "$CERT_DIR/server.key" -out "$CERT_DIR/server.csr" \
  -subj "/C=US/ST=Test/L=Test/O=Async-PHP/CN=localhost"

# Generate self-signed certificate (valid for 365 days)
openssl x509 -req -days 365 \
  -in "$CERT_DIR/server.csr" \
  -signkey "$CERT_DIR/server.key" \
  -out "$CERT_DIR/server.crt" \
  -extfile <(printf "subjectAltName=DNS:localhost,DNS:127.0.0.1,IP:127.0.0.1")

# Clean up CSR
rm "$CERT_DIR/server.csr"

echo ""
echo "Certificates generated successfully!"
echo "Certificate: $CERT_DIR/server.crt"
echo "Private Key: $CERT_DIR/server.key"
echo ""
echo "These are self-signed certificates for testing only."
echo "Clients will need to skip certificate verification (insecure mode)."
