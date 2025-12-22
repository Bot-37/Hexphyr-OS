# 🧿 Hexphyr OS

**Hexphyr OS** is a from-scratch experimental desktop operating system built with a **hybrid architecture** — a **Rust-based kernel core** combined with **C-based drivers** — designed to balance **safety, performance, and low-level control**.

This project focuses on understanding **operating system internals**, **kernel design**, and **secure system architecture**, rather than being a production-ready OS.

---

## 📌 Project Goals

- Build a **minimal yet extensible OS** from scratch
- Use **Rust** for memory safety in the kernel core
- Use **C** where low-level hardware control is required
- Learn and implement:
  - Boot process
  - Kernel initialization
  - Memory management
  - Process scheduling
  - System calls
- Maintain **clarity over complexity**

---

## 🧠 Design Philosophy

Hexphyr OS follows these core principles:

- **Security-first**: Rust is used wherever possible to reduce memory vulnerabilities
- **Explicit control**: No hidden abstractions
- **Modularity**: Clear separation between kernel, drivers, and userland
- **Educational clarity**: Code readability is prioritized over optimization tricks

---

## 🏗️ Architecture Overview

+--------------------------+ |        Userland          | |  (Shell, Utilities)     | +--------------------------+ |      System Calls        | +--------------------------+ |      Kernel Core         | |   (Written in Rust)     | +--------------------------+ |        Drivers           | |   (Written in C)        | +--------------------------+ |        Hardware          | +--------------------------+

### Why Hybrid?

- **Rust kernel** → Memory safety, concurrency guarantees
- **C drivers** → Direct hardware access and mature tooling
- Best of both worlds without overengineering

---

## 🧩 Current Features

- [x] Project structure initialized
- [x] Custom directory layout
- [x] Rust kernel scaffold
- [x] C driver interface planning
- [ ] Bootloader integration
- [ ] Basic VGA / framebuffer output
- [ ] Interrupt handling
- [ ] Memory allocator
- [ ] Process management
- [ ] Minimal shell

> ⚠️ The OS is currently **non-bootable** and under active development.

---

## 📂 Repository Structure

hexphyr-os/ ├── kernel/         # Rust kernel core │   ├── src/ │   └── Cargo.toml │ ├── drivers/        # Hardware drivers (C) │   ├── video/ │   ├── input/ │   └── storage/ │ ├── boot/           # Bootloader-related files │ ├── build/          # Build scripts and configs │ ├── docs/           # Design notes and documentation │ └── README.md

---

## 🛠️ Toolchain & Requirements

### Required Tools

- **Rust (nightly)**  
- **Cargo**
- **GCC / Clang**
- **NASM**
- **QEMU** (for testing)
- **Make**

### Target Architecture

- x86_64 (initial focus)

---

## 🔨 Build Status

🚧 **Work in Progress**

No automated build pipeline is available yet.  
Manual build steps will be added once the boot process is finalized.

---

## 🔐 Security Considerations

Hexphyr OS is designed with security awareness from day one:

- Minimal trusted code base
- Explicit memory ownership (Rust)
- No unnecessary background services
- Planned:
  - Stack protections
  - Kernel/user separation
  - Capability-based access

---

## 📖 Learning Focus

This project is primarily intended to explore:

- Kernel development
- OS boot mechanics
- Low-level memory management
- Hardware abstraction
- Secure system design

It is **not intended** to replace Linux, BSD, or existing operating systems.

---

## 🚀 Roadmap

- [ ] Bootable ISO
- [ ] Text output
- [ ] Keyboard input
- [ ] Heap allocator
- [ ] Syscall interface
- [ ] Simple shell
- [ ] User programs

---

## 🤝 Contribution

Currently a **solo research project**.  
Contributions may be considered once the kernel reaches a stable baseline.

---

## 📜 License

License will be added once the core architecture stabilizes.

---

## 🧭 Disclaimer

Hexphyr OS is an **experimental educational operating system**.  
Do **NOT** use it on real hardware.

---

**Hexphyr OS**  
_A fusion of precision, safety, and low-level control._
