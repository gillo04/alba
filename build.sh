# Compile
cargo build --release

# Create image
dd if=/dev/zero of=alba.img bs=1M count=64 
mformat -F -i alba.img ::
mmd -i alba.img ::/EFI
mmd -i alba.img ::/EFI/BOOT
mcopy -i alba.img target/x86_64-unknown-uefi/release/kernel.efi\
  ::/EFI/BOOT/BOOTX64.EFI

# Run
qemu-system-x86_64 -drive format=raw,unit=0,file=alba.img -bios /usr/share/ovmf/OVMF.fd -m 256M -vga std -name NOLIBOS -machine pc -net none
