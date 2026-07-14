MEMORY
{
  /* RP2350 (Raspberry Pi Pico 2): 4 MB external QSPI flash, 520 KB SRAM.
     Like the RP2040 it boots from external flash, but instead of a boot2 blob the
     RP2350 bootrom scans the first flash pages for a Block Loop containing an
     IMAGE_DEF metadata block. That block is the `IMAGE_DEF` static in main.rs; the
     SECTIONS below place it in `.start_block`, kept in the first 4 KB right after
     the vector table where the ROM (and picotool) look. Without it the image links
     but the ROM refuses to boot it. SRAM is 512 KB of banked contiguous RAM plus
     two 4 KB banks (SRAM8/SRAM9). Layout follows the rp-hal rp235x-hal-examples
     template; only FLASH LENGTH is raised to the Pico 2's 4 MB. */
  FLASH : ORIGIN = 0x10000000, LENGTH = 4096K
  RAM   : ORIGIN = 0x20000000, LENGTH = 512K
  SRAM8 : ORIGIN = 0x20080000, LENGTH = 4K
  SRAM9 : ORIGIN = 0x20081000, LENGTH = 4K
}

SECTIONS {
    /* Boot ROM info: right after .vector_table so the IMAGE_DEF block stays in the
       first 4 KB of flash where the ROM (and picotool) look for it. */
    .start_block : ALIGN(4)
    {
        __start_block_addr = .;
        KEEP(*(.start_block));
        KEEP(*(.boot_info));
    } > FLASH

} INSERT AFTER .vector_table;

/* Move .text to start after the boot info block. */
_stext = ADDR(.start_block) + SIZEOF(.start_block);

SECTIONS {
    /* picotool 'Binary Info' entries (empty unless the binary-info feature is on). */
    .bi_entries : ALIGN(4)
    {
        __bi_entries_start = .;
        KEEP(*(.bi_entries));
        . = ALIGN(4);
        __bi_entries_end = .;
    } > FLASH
} INSERT AFTER .text;

SECTIONS {
    /* Boot ROM extra info: after everything, so it can hold a signature. */
    .end_block : ALIGN(4)
    {
        __end_block_addr = .;
        KEEP(*(.end_block));
        __flash_binary_end = .;
    } > FLASH

} INSERT AFTER .uninit;

PROVIDE(start_to_end = __end_block_addr - __start_block_addr);
PROVIDE(end_to_start = __start_block_addr - __end_block_addr);
