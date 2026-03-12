/*
 * drivers/serial/serial.h  —  8250/16550-compatible UART driver
 *
 * Provides a minimal, polling-mode serial port driver for use by C userland
 * or early-boot C code.  The kernel itself uses the Rust serial module; this
 * file exists so that C drivers and userland utilities can share a single
 * serial output path without depending on the Rust runtime.
 *
 * Security notes:
 *   - write_serial_str() accepts only a null-terminated string pointer; the
 *     length is bounded internally.
 *   - No buffer read-back (RX) is exposed here: receiving raw bytes from an
 *     unvalidated serial channel is left to higher-level code.
 */

#ifndef HEXPHYR_SERIAL_H
#define HEXPHYR_SERIAL_H

#include <stdint.h>

/* I/O port base address for COM1. */
#define SERIAL_COM1_BASE  0x3F8U

/* Initialise the UART at the given I/O base address.
 * Must be called before any other serial function.
 * Returns 0 on success, -1 if the loopback self-test fails (hardware absent).
 */
int serial_init(uint16_t port);

/* Write a single byte to the UART.  Blocks until the transmit-hold register
 * is empty (polling). */
void serial_write_byte(uint16_t port, uint8_t byte);

/* Write a null-terminated string.  Each '\n' is expanded to "\r\n" for
 * compatibility with terminal emulators. */
void serial_write_str(uint16_t port, const char *str);

#endif /* HEXPHYR_SERIAL_H */
