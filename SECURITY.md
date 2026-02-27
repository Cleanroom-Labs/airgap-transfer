# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | ✓         |

## Reporting a Vulnerability

**Please do not open a public GitHub issue for security vulnerabilities.**

Use GitHub's [private vulnerability reporting](../../security/advisories/new) feature to
report vulnerabilities confidentially. This ensures the issue can be assessed and patched
before public disclosure.

You can expect an acknowledgement within **3 business days** and a resolution plan within
**14 days** of confirmed reproduction. Critical vulnerabilities (remote code execution,
data loss, or privilege escalation) are prioritised.

## Scope

The following are considered security vulnerabilities for this project:

- Arbitrary code execution during pack/unpack operations
- Path traversal or directory escape from the destination path
- Checksum bypass or integrity verification failure
- Information disclosure from manifest files or chunk metadata

Crashes, assertion failures, and panics on malformed input are treated as bugs, not
security vulnerabilities, unless they can be triggered remotely or used to exfiltrate data.
