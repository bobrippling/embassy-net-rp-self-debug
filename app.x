MEMORY {
    /*  app code lives after the bootloader and we don't touch
     *  the bootloader itself, or boot2 */

    FLASH
    : ORIGIN = 0x10000000 + 0x100 + 4k
    , LENGTH = 2048K - 0x100 - 4k

    RAM : ORIGIN = 0x20000000, LENGTH = 256K /* 264k? */
}

SECTIONS {
    // WIP
    .text
}

_user_main = 0x100005e8 /* ORIGIN(FLASH) + LENGTH(FLASH) / * user's main must be at end of bootloader flash, start of user flash */

INSERT AFTER .text;
