# Compile source
all:
	cd kernel && cargo build --release
	cd user1 && cargo build --release
	cd user2 && cargo build --release
	cd gui_demo && cargo build --release
	cd desktop && cargo build --release


# Build image
img: all
	dd if=/dev/zero of=alba.img bs=1M count=64 
	mformat -F -i alba.img ::
	mmd -i alba.img ::/EFI
	mmd -i alba.img ::/EFI/BOOT
	mcopy -i alba.img kernel/target/x86_64-unknown-uefi/release/kernel.efi ::/EFI/BOOT/BOOTX64.EFI

	mmd -i alba.img ::/USER
	mcopy -i alba.img user1/target/x86_64-unknown-none/release/user1 ::/USER/USER1
	mcopy -i alba.img user2/target/x86_64-unknown-none/release/user2 ::/USER/USER2
	mcopy -i alba.img gui_demo/target/x86_64-unknown-none/release/gui_demo ::/USER/GUI_DEMO
	mcopy -i alba.img desktop/target/x86_64-unknown-none/release/desktop ::/USER/DESKTOP
	mcopy -i alba.img logo/alba_logo.ppm ::/USER/LOGO.PPM
	mcopy -i alba.img assets/pointer.ppm ::/USER/POINTER.PPM
	mcopy -i alba.img assets/zap-light16.psf ::/USER/FONT.PSF
	mcopy -i alba.img assets/exe_icon.ppm ::/USER/EXE_ICON.PPM

# Run in qemu
run: img 
	qemu-system-x86_64 -drive format=raw,unit=0,file=alba.img -bios /usr/share/ovmf/OVMF.fd -m 256M -vga std -name Alba -machine pc -net none

run-int:
	qemu-system-x86_64 -d int -drive format=raw,unit=0,file=alba.img -bios /usr/share/ovmf/OVMF.fd -m 256M -vga std -name Alba -machine pc -net none
