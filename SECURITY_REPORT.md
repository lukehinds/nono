### Security Analysis Report for `nono`

This report summarizes the findings of a security analysis of the `nono` codebase, focusing on its sandboxing capabilities for macOS (Seatbelt) and Linux (Landlock).

### Executive Summary

`nono` has a well-designed architecture, featuring a robust configuration system with signature verification and downgrade protection. Its sandboxing implementation on **Linux (via Landlock) is strong, modern, and secure.** The **macOS (via Seatbelt) implementation has been significantly improved** by addressing a critical vulnerability regarding global file read access. Both platforms still lack comprehensive process isolation, which presents a risk for secret leakage.

---

### 1. macOS (Seatbelt) Implementation Analysis: **Improved, but Major Weaknesses Remain**

The macOS sandbox has been improved to address the critical global read access vulnerability. However, other significant weaknesses persist.

**Addressed Critical Vulnerability:**
*   **Global Filesystem Read Access:** The previous critical vulnerability where the Seatbelt profile started with a global `(allow file-read*)` has been successfully addressed. The profile now correctly operates on a pure allowlist model for file reads, only granting access to explicitly specified paths (minimal `exec()` permissions, system paths, and user-granted paths). This significantly improves the security posture for file read operations.

**Major Weaknesses:**
*   **No Process Isolation:** The profile uses `(allow process*)` and does not deny `process-info*`. This allows the sandboxed process to inspect other processes running as the same user (e.g., by running `ps aux`). This is a major information leak vector and can be used to steal secrets from the command-line arguments or environment of other programs.
*   **Overly Permissive Network Access:** The default network policy allows the process to make outbound connections, bind to ports, and accept inbound connections (`allow network-outbound`, `network-inbound`, `network-bind`). Granting inbound and bind access by default is unnecessary and increases the attack surface.
*   **Overly Broad System Rules:** Blanket `(allow system*)`, `(allow mach*)`, and `(allow ipc*)` rules grant far more permissions than required, significantly weakening the "default-deny" security posture.

---

### 2. Linux (Landlock) Implementation Analysis: **Strong**

The Linux implementation is a model of modern sandboxing and is fundamentally secure from a filesystem perspective.

**Strengths:**
*   **Pure Allowlist Model:** The sandbox correctly operates on a "default-deny" basis. It starts with zero permissions and only adds `allow` rules for paths explicitly granted by the user or required by the system. This is the most secure sandboxing strategy.
*   **Robust Protection Against Deletion:** The implementation cleverly grants write permissions *without* granting file/directory deletion permissions at the syscall level. This provides powerful, OS-enforced protection against destructive actions like `rm -rf`, even inside a writable working directory.

**Weaknesses:**
*   **No Process Isolation:** Similar to the macOS implementation, the Landlock sandbox only restricts filesystem and (optionally) network access. It **does not** prevent the sandboxed process from using the `/proc` filesystem to inspect other processes owned by the same user. This presents the same risk of information and secret leakage from other processes.

---

### 3. General Codebase & Configuration Issues

*   **Placeholder Signing Key:** The public key used for signature verification (`AUTHOR_PUBLIC_KEY`) is a placeholder. **This means no signature verification is currently active**, rendering the trust model for profiles and security lists ineffective.
*   **Secrets in Environment Variables:** Passing secrets to the sandboxed process via environment variables is a common but risky practice. Given the lack of process isolation on both platforms, a malicious agent could inspect the environment of other running processes, or its own environment could be inspected by another un-sandboxed process.

---

### 4. Recommendations

**High-Priority (Address Before Use):**
1.  **Implement Process Isolation:**
    *   **macOS:** Add `(deny process-info*)`, `(deny process-list-all)`, and related rules to the Seatbelt profile to prevent inspection of other processes.
    *   **Linux:** To achieve true process isolation, Landlock must be combined with PID namespaces.
2.  **Replace Placeholder Public Key:** Generate a real `minisign` keypair and embed the public key in the binary to enable signature verification.

**Medium-Priority:**
3.  **Harden Network Rules:** Change the default network `allow` on macOS to be `(allow network-outbound)` only.
4.  **Harden macOS System Rules:** Replace broad `(allow system*)` and `(allow mach*)` rules with a minimal, curated list of specific `allow` rules required for agents to function.
5.  **Clarify Deletion Protection (macOS):** The documentation and/or implementation for file deletion protection on macOS should be clarified to reflect that the `(deny file-write-unlink)` is often overridden for user-writable directories.

**Conclusion:** The project has a strong foundation and an excellent Linux implementation. The critical filesystem read vulnerability on macOS has been addressed. Focusing on implementing process isolation and enabling signature verification are the next most important steps to elevate the overall security posture of `nono`.