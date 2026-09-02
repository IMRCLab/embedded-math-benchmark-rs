MEMORY
{
  /* nRF52840: 1 MB Flash, 256 KB RAM. No SoftDevice, so flash starts at 0. */
  FLASH : ORIGIN = 0x00000000, LENGTH = 1024K
  RAM   : ORIGIN = 0x20000000, LENGTH = 256K
}
