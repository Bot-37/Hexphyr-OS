/*
 * drivers/pci/pci.h  —  PCI Configuration Space access (Type-1 mechanism)
 *
 * Provides a read-only enumeration interface for PCI bus topology and
 * configuration registers.  Write access is intentionally not exposed at this
 * level; device-specific drivers should own all PCI configuration writes.
 *
 * Security notes:
 *   - All port I/O is centralised here; no other driver should issue raw
 *     IN/OUT to the PCI config address ports (0xCF8/0xCFC).
 *   - Returned vendor/device IDs are validated before being trusted.
 */

#ifndef HEXPHYR_PCI_H
#define HEXPHYR_PCI_H

#include <stdint.h>

/* Maximum PCI topology dimensions. */
#define PCI_MAX_BUS   256U
#define PCI_MAX_SLOT   32U
#define PCI_MAX_FUNC    8U

/* Sentinel for an absent device (vendor == 0xFFFF). */
#define PCI_VENDOR_NONE  0xFFFFU

/*
 * Snapshot of the most useful fields from PCI Configuration Space header
 * type 0 (general device).
 */
typedef struct {
    uint8_t  bus;
    uint8_t  slot;
    uint8_t  function;
    uint16_t vendor_id;
    uint16_t device_id;
    uint8_t  class_code;
    uint8_t  subclass;
    uint8_t  prog_if;
    uint8_t  revision_id;
    uint8_t  header_type;
    uint8_t  interrupt_line;
    uint8_t  interrupt_pin;
} pci_device_t;

/* Callback type for pci_enumerate().  Return non-zero to stop enumeration. */
typedef int (*pci_enum_cb_t)(const pci_device_t *dev, void *user);

/*
 * Read a 32-bit dword from PCI Configuration Space.
 *   bus      0–255
 *   slot     0–31
 *   func     0–7
 *   offset   byte offset into config space; must be 4-byte aligned.
 */
uint32_t pci_read32(uint8_t bus, uint8_t slot, uint8_t func, uint8_t offset);

/* Read 16-bit word; offset must be 2-byte aligned. */
uint16_t pci_read16(uint8_t bus, uint8_t slot, uint8_t func, uint8_t offset);

/* Read 8-bit byte. */
uint8_t pci_read8(uint8_t bus, uint8_t slot, uint8_t func, uint8_t offset);

/*
 * Enumerate all present PCI devices.  For every device found, `cb` is called
 * with a pointer to a temporary pci_device_t.  Enumeration stops early if `cb`
 * returns non-zero.
 */
void pci_enumerate(pci_enum_cb_t cb, void *user);

#endif /* HEXPHYR_PCI_H */
