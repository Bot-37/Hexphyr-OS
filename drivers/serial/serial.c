/*
 * drivers/serial/serial.c  —  8250/16550-compatible UART driver
 *
 * Polling-mode implementation.  Does NOT use interrupts or DMA.
 * Suitable for early-boot debug output and userland serial utilities.
 */

#include "serial.h"

/* ---------------------------------------------------------------------------
 * Low-level I/O port helpers.
 * In a freestanding (no-libc) environment these inline the IN/OUT instructions
 * directly.  When compiled for a hosted test harness, they can be mocked.
 * -------------------------------------------------------------------------*/

static inline void outb(uint16_t port, uint8_t value)
{
    __asm__ volatile ("outb %0, %1" : : "a"(value), "Nd"(port) : "memory");
}

static inline uint8_t inb(uint16_t port)
{
    uint8_t value;
    __asm__ volatile ("inb %1, %0" : "=a"(value) : "Nd"(port) : "memory");
    return value;
}

/* ---------------------------------------------------------------------------
 * Register offsets relative to the UART base address.
 * -------------------------------------------------------------------------*/
#define UART_THR   0   /* Transmit Holding Register  (write, DLAB=0) */
#define UART_IER   1   /* Interrupt Enable Register  (write, DLAB=0) */
#define UART_DLL   0   /* Divisor Latch Low byte     (write, DLAB=1) */
#define UART_DLH   1   /* Divisor Latch High byte    (write, DLAB=1) */
#define UART_FCR   2   /* FIFO Control Register      (write)         */
#define UART_LCR   3   /* Line Control Register      (write)         */
#define UART_MCR   4   /* Modem Control Register     (write)         */
#define UART_LSR   5   /* Line Status Register       (read)          */

/* LSR bits */
#define LSR_THRE   0x20  /* Transmit-Hold-Register Empty */
#define LSR_DR     0x01  /* Data Ready                   */

/* LCR bits */
#define LCR_DLAB   0x80  /* Divisor Latch Access Bit     */
#define LCR_8N1    0x03  /* 8 data bits, no parity, 1 stop bit */

/* MCR bits */
#define MCR_DTR    0x01
#define MCR_RTS    0x02
#define MCR_OUT2   0x08
#define MCR_LOOP   0x10  /* Loopback mode */

int serial_init(uint16_t port)
{
    /* Disable all interrupts. */
    outb(port + UART_IER, 0x00);

    /* Set baud rate to 38400 (divisor = 3 at 1.8432 MHz clock). */
    outb(port + UART_LCR, LCR_DLAB);
    outb(port + UART_DLL, 0x03);  /* low byte  */
    outb(port + UART_DLH, 0x00);  /* high byte */

    /* 8N1, DLAB cleared. */
    outb(port + UART_LCR, LCR_8N1);

    /* Enable and clear FIFOs, set 14-byte interrupt threshold. */
    outb(port + UART_FCR, 0xC7);

    /* Enable loopback for self-test. */
    outb(port + UART_MCR, MCR_LOOP | MCR_RTS | MCR_DTR);

    /* Send a known byte and verify it echoes back. */
    outb(port + UART_THR, 0xAE);
    if (inb(port + UART_THR) != 0xAE) {
        /* Hardware absent or not functioning correctly. */
        return -1;
    }

    /* Self-test passed.  Disable loopback, enable normal operation. */
    outb(port + UART_MCR, MCR_DTR | MCR_RTS | MCR_OUT2);
    return 0;
}

void serial_write_byte(uint16_t port, uint8_t byte)
{
    /* Poll until the transmit-hold register is empty. */
    while ((inb(port + UART_LSR) & LSR_THRE) == 0)
        __asm__ volatile ("pause" ::: "memory");

    outb(port + UART_THR, byte);
}

void serial_write_str(uint16_t port, const char *str)
{
    if (!str) return;

    while (*str) {
        /* Expand bare LF to CR LF for terminal compatibility. */
        if (*str == '\n') {
            serial_write_byte(port, '\r');
        }
        serial_write_byte(port, (uint8_t)*str);
        str++;
    }
}
