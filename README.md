# Alba OS
Alba OS is a 64 bit x86_64 multitasking operating system written in Rust.

![Alba OS logo](./logo/alba_logo.png)

## Planned features
- [x] Framebuffer console
- [x] Virtual memory map
- [x] Heap
- [x] ATA PIO driver
- [x] Keyboard driver
- [x] FAT32 file system
- [x] Time based preemptive multitasking
- [x] File system and graphic and  system calls
- [ ] Process creation and management system calls
- [ ] Sleep system call
- [ ] Mouse driver
- [ ] Full GUI library
- [ ] Desktop environment
- [ ] USB mass storage device driver

## Building and running
To build a disk image of the OS, run the `build.sh` script at the root of the project:

    sh build.sh

This will create a `alba.img` file at the root of the project and open quemu to run it.

If you want to test this on real hardware, you can flash the image on a USB stick and boot from it, but be ware: since the OS does not yet support a USB driver, when running it on real hardware from a USB stick it will panic when trying to initialize the ATA PIO driver (or in case an ATA bus exists, it will read garbage data off the disk).
