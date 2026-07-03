MEMORY
{
  /* RP2040 (Raspberry Pi Pico 1): 2 MB external QSPI flash, 264 KB SRAM.
     The RP2040 has no internal program flash; it boots from external flash via a
     256-byte second-stage bootloader (boot2) that must live at the very start of
     flash. The boot2 blob itself is provided by rp-pico's default `boot2` feature
     (symbol BOOT2_FIRMWARE); the SECTIONS block below places it. */
  BOOT2 : ORIGIN = 0x10000000, LENGTH = 0x100
  FLASH : ORIGIN = 0x10000100, LENGTH = 2048K - 0x100
  RAM   : ORIGIN = 0x20000000, LENGTH = 264K
}

EXTERN(BOOT2_FIRMWARE)

SECTIONS {
  /* Place the boot2 blob at the start of flash, ahead of .text. */
  .boot2 ORIGIN(BOOT2) :
  {
    KEEP(*(.boot2));
  } > BOOT2
} INSERT BEFORE .text;
