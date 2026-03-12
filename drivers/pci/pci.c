/*
 * drivers/pci/pci.c  —  PCI Configuration Space access (Type-1 / CAM mechanism)
 *
 * Uses the standard legacy I/O-port mechanism:
 *   0xCF8  CONFIG_ADDRESS  (32-bit R/W)
 *   0xCFC  CONFIG_DATA     (32-bit R/W)
 *
 * Reference: PCI Local Bus Specification Rev 3.0, §3.2.2.3.2.
 */

#include "pci.h"

/* I/O port addresses for the PCI configuration mechanism. */
#define PCI_CONFIG_ADDRESS  0x0CF8U
#define PCI_CONFIG_DATA     0x0CFCU

/* Bit 31 of CONFIG_ADDRESS must be set to enable the cycle. */
#define PCI_ENABLE_BIT      (1U << 31)

/* ---------------------------------------------------------------------------
 * Port I/O primitives
 * -------------------------------------------------------------------------*/

static inline void outl(uint16_t port, uint32_t value)
{
    __asm__ volatile ("outl %0, %1" : : "a"(value), "Nd"(port) : "memory");
}

static inline uint32_t inl(uint16_t port)
{
    uint32_t value;
    __asm__ volatile ("inl %1, %0" : "=a"(value) : "Nd"(port) : "memory");
    return value;
}

static inline void outb_pci(uint16_t port, uint8_t value)
{
    __asm__ volatile ("outb %0, %1" : : "a"(value), "Nd"(port) : "memory");
}

static inline uint16_t inw(uint16_t port)
{
    uint16_t value;
    __asm__ volatile ("inw %1, %0" : "=a"(value) : "Nd"(port) : "memory");
    return value;
}

static inline uint8_t inb_pci(uint16_t port)
{
    uint8_t value;
    __asm__ volatile ("inb %1, %0" : "=a"(value) : "Nd"(port) : "memory");
    return value;
}
(void)outb_pci; /* suppress unused-function warning */

/* ---------------------------------------------------------------------------
 * PCI configuration address encoding
 *
 * Bits [31]    : Enable bit (must be 1)
 * Bits [23:16] : Bus number
 * Bits [15:11] : Device (slot) number
 * Bits [10:8]  : Function number
 * Bits [7:2]   : Register number (dword index)
 * Bits [1:0]   : Must be 00
 * -------------------------------------------------------------------------*/

static inline uint32_t pci_address(uint8_t bus, uint8_t slot,
                                    uint8_t func, uint8_t offset)
{
    return PCI_ENABLE_BIT
         | ((uint32_t)bus  << 16)
         | ((uint32_t)(slot & 0x1FU) << 11)
         | ((uint32_t)(func & 0x07U) <<  8)
         | ((uint32_t)(offset & 0xFCU));  /* mask low 2 bits: dword aligned */
}

uint32_t pci_read32(uint8_t bus, uint8_t slot, uint8_t func, uint8_t offset)
{
    outl(PCI_CONFIG_ADDRESS, pci_address(bus, slot, func, offset));
    return inl(PCI_CONFIG_DATA);
}

uint16_t pci_read16(uint8_t bus, uint8_t slot, uint8_t func, uint8_t offset)
{
    uint32_t dword = pci_read32(bus, slot, func, offset & 0xFCU);
    /* Select the correct 16-bit half based on the low bit of the offset. */
    uint8_t shift = (offset & 2U) * 8U;
    return (uint16_t)((dword >> shift) & 0xFFFFU);
}

uint8_t pci_read8(uint8_t bus, uint8_t slot, uint8_t func, uint8_t offset)
{
    uint32_t dword = pci_read32(bus, slot, func, offset & 0xFCU);
    uint8_t  shift = (offset & 3U) * 8U;
    return (uint8_t)((dword >> shift) & 0xFFU);
}

void pci_enumerate(pci_enum_cb_t cb, void *user)
{
    for (uint16_t bus = 0; bus < PCI_MAX_BUS; bus++) {
        for (uint8_t slot = 0; slot < PCI_MAX_SLOT; slot++) {
            for (uint8_t func = 0; func < PCI_MAX_FUNC; func++) {
                uint16_t vendor_id = pci_read16(
                    (uint8_t)bus, slot, func, 0x00);

                /* 0xFFFF means no device present. */
                if (vendor_id == PCI_VENDOR_NONE) {
                    /* Skip remaining functions for single-function devices. */
                    if (func == 0) break;
                    continue;
                }

                pci_device_t dev;
                dev.bus         = (uint8_t)bus;
                dev.slot        = slot;
                dev.function    = func;
                dev.vendor_id   = vendor_id;
                dev.device_id   = pci_read16((uint8_t)bus, slot, func, 0x02);

                uint32_t class_rev = pci_read32((uint8_t)bus, slot, func, 0x08);
                dev.revision_id = (uint8_t)(class_rev & 0xFFU);
                dev.prog_if     = (uint8_t)((class_rev >>  8) & 0xFFU);
                dev.subclass    = (uint8_t)((class_rev >> 16) & 0xFFU);
                dev.class_code  = (uint8_t)((class_rev >> 24) & 0xFFU);

                uint16_t hdr_word = pci_read16((uint8_t)bus, slot, func, 0x0E);
                dev.header_type = (uint8_t)(hdr_word & 0xFFU);

                uint16_t irq_word = pci_read16((uint8_t)bus, slot, func, 0x3C);
                dev.interrupt_line = (uint8_t)(irq_word & 0xFFU);
                dev.interrupt_pin  = (uint8_t)((irq_word >> 8) & 0xFFU);

                if (cb(&dev, user) != 0) return;

                /* Bit 7 of header_type: multi-function device flag.         */
                /* If not set and we are at function 0, skip remaining funcs. */
                if (func == 0 && (dev.header_type & 0x80U) == 0) break;
            }
        }
    }
}
