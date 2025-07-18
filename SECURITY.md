# Security Policy

## Supported Versions

We release patches for security vulnerabilities for the following versions:

| Version | Supported          |
| ------- | ------------------ |
| 0.5.x   | :white_check_mark: |
| < 0.5   | :x:                |

## Reporting a Vulnerability

We take the security of CIM-IPLD seriously. If you believe you have found a security vulnerability, please report it to us as described below.

### Please do NOT:
- Open a public issue
- Disclose the vulnerability publicly before it has been addressed

### Please DO:
- Email your findings to [INSERT SECURITY EMAIL]
- Include detailed steps to reproduce the vulnerability
- Include the version of CIM-IPLD affected
- Include any relevant logs or screenshots

### What to expect:
- Acknowledgment of your report within 48 hours
- Regular updates on the progress of addressing the vulnerability
- Credit for responsible disclosure (if desired)

## Security Best Practices

When using CIM-IPLD:

### 1. Keep Dependencies Updated
```bash
cargo update
cargo audit
```

### 2. Use Encryption for Sensitive Data
```rust
use cim_ipld::content_types::encryption::{ContentEncryption, EncryptionAlgorithm};

let key = ContentEncryption::generate_key(EncryptionAlgorithm::ChaCha20Poly1305);
let encryption = ContentEncryption::new(key, EncryptionAlgorithm::ChaCha20Poly1305)?;
```

### 3. Validate Content Types
```rust
// Always verify content before processing
if !PdfDocument::verify(&untrusted_data) {
    return Err("Invalid PDF content");
}
```

### 4. Secure NATS Configuration
- Use TLS for NATS connections in production
- Enable authentication and authorization
- Restrict access to JetStream buckets

### 5. Access Control
- Implement application-level access control
- Use domain partitioning for sensitive content
- Audit access to sensitive domains

## Known Security Considerations

1. **CID Integrity**: CIDs provide cryptographic integrity but not confidentiality
2. **Network Security**: Use TLS for all network connections
3. **Key Management**: Store encryption keys securely
4. **Input Validation**: Always validate untrusted input

---
Copyright 2025 Cowboy AI, LLC.