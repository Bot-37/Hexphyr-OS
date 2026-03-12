<<<<<<< HEAD
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
=======
# Hexphyr OS

A 64-bit, x86-64 operating system kernel written primarily in **Rust** with
assembly bootstrap and C driver/userland stubs.

---

## Architecture

| Layer | Technology | Location |
|---|---|---|
| UEFI Bootloader | Rust (`x86_64-unknown-uefi`) | `bootloader/` |
| Kernel | Rust (`x86_64-unknown-none`) | `kernel/` |
| Drivers | C | `drivers/` |
| Userland | C | `userland/` |

The normal boot path uses **GRUB** with the **Multiboot2** protocol to load
the kernel ELF directly.  A separate UEFI bootloader (`bootloader/`) is the
foundation for a future native UEFI boot path.

---

## Security posture

### x86-64 CPU hardening (boot.s)
| Feature | Bit | Effect |
|---|---|---|
| `EFER.NXE` | bit 11 | Marks data pages non-executable (No-Execute) |
| `CR0.WP` | bit 16 | Prevents kernel from writing read-only pages (Write Protect) |
| `CR4.SMEP` | bit 20 | Kernel cannot execute user-space pages (CPUID-gated) |
| `CR4.SMAP` | bit 21 | Kernel cannot access user-space pages without STAC/CLAC (CPUID-gated) |
| `CR4.UMIP` | bit 11 | Blocks `SGDT`/`SIDT`/`SLDT` in user mode (CPUID-gated) |

SMEP, SMAP, and UMIP are checked via `CPUID` leaf 7 in the bootstrap assembly
before being set, so the kernel runs on older hardware that lacks them.

### Kernel hardening
- **Explicit BSS zeroing** before page-table construction (defense-in-depth;
  GRUB also zeroes BSS per the Multiboot2 spec).
- **Serial logger deadlock fix**: the `spin::Mutex` is acquired exactly once
  per log call — the original code acquired it twice, which would deadlock on
  any reentrant log from within a lock-holding context.
- **Serial loopback self-test**: `serial_init()` tests the hardware before
  committing to it; a missing/broken UART is handled gracefully.
- **GDT + TSS with IST**: the double-fault handler runs on a dedicated 20 KiB
  IST stack (`gdt.rs`).  A corrupted kernel stack that causes a double fault
  will now produce a serial log entry instead of silently triple-faulting.
- **Full IDT**: 20 exception vectors covered with structured serial-log output
  before halting; breakpoints are resumable.
- **CR2 read on page fault**: captured atomically before any other memory
  access can overwrite it.
- **Multiboot2 parser hardened**: all multi-byte reads use `ptr::read_unaligned`
  (eliminates Rust UB), pointer alignment is checked (8-byte spec requirement),
  and the reserved field is validated before any tags are iterated.
- **Overflow checks**: enabled in both `dev` and `release` profiles.
- **Debug-info stripped** from release builds (`strip = "debuginfo"`).

---

## Building

### Prerequisites
```sh
# Rust toolchain (nightly required for abi_x86_interrupt)
rustup toolchain install nightly
rustup target add x86_64-unknown-none --toolchain nightly
rustup target add x86_64-unknown-uefi --toolchain nightly

# Build tools
sudo apt-get install grub-common grub-pc-bin xorriso mtools qemu-system-x86
```

### Quick build
```sh
# Build kernel only (debug)
cd kernel && cargo +nightly build

# Build kernel (release)
cd kernel && cargo +nightly build --release

# Build full bootable ISO (kernel + UEFI bootloader + GRUB)
make all
```

### Run in QEMU (GRUB/Multiboot2 path)
```sh
cd kernel && ./build.sh   # produces kernel/hexphyr.iso
./run.sh
```

Or use the top-level Makefile target:
```sh
make run   # requires OVMF firmware at /usr/share/ovmf/
```

### Docker simulation
```sh
./tools/docker-sim.sh
```

---

## Layout

```
bootloader/    Rust UEFI bootloader (foundation; kernel load TODO)
drivers/
  pci/         PCI Configuration Space bus enumeration (C)
  serial/      8250/16550 UART driver (C, polling mode)
kernel/
  src/
    boot.s     Multiboot2 entry, BSS zero, long-mode setup, security bits
    gdt.rs     GDT + TSS with IST-0 for double-fault handler
    interrupts.rs  Full IDT — 20 exception vectors
    memory.rs  Lock-free bump-pointer physical frame allocator
    multiboot.rs   Hardened Multiboot2 info-structure parser
    serial.rs  Rust serial logger (COM1, deadlock-safe)
    gui.rs     Direct-write framebuffer renderer
    main.rs    Kernel entry point
  linker.ld    Kernel linker script
userland/
  init/init.c  PID-1 init process skeleton
  sh/smallsh.c Minimal interactive shell skeleton
```

---

## Toolchain

The project pins **Rust nightly** via `rust-toolchain.toml`.  Nightly is
needed for `#![feature(abi_x86_interrupt)]`, which enables the
`extern "x86-interrupt"` calling convention required by all IDT handlers.


Hexphyr is an experimental x86_64 kernel with a Multiboot2 boot path and a minimal framebuffer GUI.

## Local Simulation

Run directly on the host:

```bash
cd kernel
./build.sh
./run.sh
```

Headless serial-only mode:

```bash
HEXPHYR_HEADLESS=1 ./run.sh
```

GUI mode (requires a working local display server):

```bash
HEXPHYR_HEADLESS=0 ./run.sh
```

## Docker Simulation

Build the image:

```bash
docker build -t hexphyr-os:dev .
```

Build and run simulation in one step:

```bash
./tools/docker-sim.sh
```

Run simulation directly from repo root (headless by default):

```bash
SIM_TIMEOUT_SEC=25 ./tools/simulate-kernel.sh
```

Or via Make:

```bash
make docker-sim
```

`tools/docker-sim.sh` and `tools/simulate-kernel.sh` run QEMU headless by default (`HEXPHYR_HEADLESS=1`) and exit after `SIM_TIMEOUT_SEC` seconds (default `25`).
>>>>>>> 877ab78 (Add Docker support and enhance kernel simulation)
