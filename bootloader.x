MEMORY {
    BOOT2 : ORIGIN = 0x10000000, LENGTH = 0x100

    /* our code lives untouched at BOOTLOADER,
     * user code lives in the FLASH region (hardcoded in cortex-m-rt)
     *
     * we then flash the bootloader once and subsequent flashes flash FLASH
     */
    FLASH
    : ORIGIN = ORIGIN(BOOT2) + LENGTH(BOOT2)
    , LENGTH = 32k

    RAM : ORIGIN = 0x20000000, LENGTH = 256K /* 264k? */
}

EXTERN(BOOT2_FIRMWARE)

SECTIONS {
    /* ### Boot loader */
    .boot2 ORIGIN(BOOT2) :
    {
        KEEP(*(.boot2));
    } > BOOT2
}
INSERT BEFORE .text;

_user_main = 0x100005e8; /* ORIGIN(FLASH) + LENGTH(FLASH) / * user's main must be at end of bootloader flash, start of user flash */
