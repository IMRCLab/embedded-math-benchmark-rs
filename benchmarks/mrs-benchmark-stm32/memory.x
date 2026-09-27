MEMORY
{
  /* STM32F405RGT6: 1 MB Flash, 128 KB SRAM1+SRAM2 */
  FLASH : ORIGIN = 0x08000000, LENGTH = 1024K
  RAM   : ORIGIN = 0x20000000, LENGTH = 128K
}
/* crazyflie-fw kalman_core statics; placed after .bss so cortex-m-rt zeroes them, see docs/c-suites.md */
SECTIONS {
  .ccmbss (NOLOAD) : ALIGN(4)
  {
    *(.ccmbss .ccmbss.*);
    . = ALIGN(4);
  } > RAM
} INSERT AFTER .bss;
