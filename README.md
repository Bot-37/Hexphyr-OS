# Hexphyr OS — Repository Documentation

This document describes the **actual implemented work**, **current structure**, and **intended execution flow** of the Hexphyr OS repository.

This is a **technical documentation file**, meant for:
- Developers
- Reviewers
- Future contributors
- The project owner (single-maintainer model)

---

## 1. Repository Purpose

The Hexphyr OS repository exists to:

- Serve as the **source of truth** for the Hexphyr OS codebase
- Track incremental progress of a **from-scratch operating system**
- Maintain a **clean separation** between kernel, drivers, boot logic, and documentation
- Enable future CI/CD and automated testing without structural refactors

This repository is designed to be **production-friendly**, meaning:
- Predictable structure
- Clear ownership of responsibilities
- No experimental code mixed with core logic
- No hidden build steps

---

## 2. Current Development State

| Component        | Status        |
|------------------|---------------|
| Kernel scaffold  | Implemented   |
| Driver layout    | Implemented   |
| Boot logic       | Planned       |
| Userland         | Planned       |
| Build system     | Partial       |
| CI/CD            | Not added     |

⚠️ The OS is **not bootable yet**.  
All current work is **foundation-level**.

---

## 3. Directory Structure (Authoritative)

Each directory has **one responsibility only**.

---

## 4. Kernel (`/kernel`)

### Purpose
The `kernel/` directory contains the **core operating system logic**, written in **Rust**.

Rust is chosen to:
- Prevent memory corruption
- Enforce ownership rules
- Reduce kernel-level vulnerabilities
- Enable safer concurrency later

### Current State
- Kernel crate initialized
- Entry points defined
- No hardware access yet
- No memory allocator yet

### Responsibilities
The kernel will eventually handle:
- CPU initialization
- Memory management
- Interrupt handling
- System calls
- Process scheduling

### What is *not* allowed here
- Hardware-specific code
- Direct I/O port manipulation
- Architecture-specific assembly (kept in `/boot`)

---

## 5. Drivers (`/drivers`)

### Purpose
The `drivers/` directory contains **hardware drivers written in C**.

C is used here intentionally because:
- Direct hardware access is required
- Existing documentation and patterns are mature
- ABI compatibility is predictable

### Planned Driver Categories

### Design Rule
Drivers:
- Do NOT manage memory independently
- Communicate with the kernel via **explicit interfaces**
- Never call kernel internals directly

This avoids tight coupling.

---

## 6. Boot (`/boot`)

### Purpose
Contains all logic required to:
- Start execution from firmware
- Load the kernel
- Transfer control safely

### Planned Responsibilities
- Bootloader configuration
- Assembly stubs
- Architecture initialization (x86_64)

### Why separated?
Boot code:
- Is architecture-specific
- Requires assembly
- Should never contaminate kernel logic

---

## 7. Build System (`/build`)

### Purpose
Centralized location for:
- Build scripts
- Linker scripts
- Target configurations

### Design Goal
The build system must:
- Be deterministic
- Be reproducible
- Support CI in the future
- Avoid ad-hoc shell hacks

No implicit steps are allowed.

---

## 8. Documentation (`/docs`)

### Purpose
Holds **non-code knowledge**, including:
- Architecture decisions
- Design tradeoffs
- Future planning
- This document

### Rule
If something is **important to remember**, it goes in `/docs`, not comments.

---

## 9. Coding Standards

### Kernel (Rust)
- No `unsafe` unless absolutely required
- Unsafe blocks must be documented
- Explicit lifetimes preferred
- No macros without justification

### Drivers (C)
- No global mutable state
- No magic numbers
- Hardware registers must be documented

---

## 10. Security Model (Planned)

Hexphyr OS is designed with **security as a first-class concern**.

Planned measures:
- Kernel/User separation
- No shared writable memory
- Minimal attack surface
- Capability-based access (later stage)

No feature is accepted if it weakens isolation.

---

## 11. Contribution Model

Currently:
- **Single maintainer**
- No external contributions accepted

Future:
- Clear contribution rules
- Mandatory documentation for any feature
- Code review required

---

## 12. Production Readiness Policy

A component is considered **production-ready** only if:
- It is documented
- It has deterministic behavior
- It does not rely on undefined behavior
- It passes emulator testing

---

## 13. Explicit Non-Goals

Hexphyr OS does **NOT** aim to:
- Replace Linux
- Be daily-driver usable
- Compete with existing kernels

The goal is **control, understanding, and correctness**.

---

## 14. Ownership

Hexphyr OS is maintained by its creator.  
All architectural decisions are centralized to prevent fragmentation.

---

## 15. Final Note

This document must be updated **every time**:
- A new subsystem is added
- A design decision changes
- A responsibility shifts

If the documentation is outdated, the repository is considered **invalid**.

