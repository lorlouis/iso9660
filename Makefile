STAGE1 = boot/stage1.asm
STAGE1_BIN = stage1.bin

ISO_FILE = mvb.iso

ISO_DATA = iso.bin

QEMU = qemu-system-i386


run: $(ISO_FILE)
	$(QEMU) -boot d -cdrom $(ISO_FILE) -m 512

$(ISO_DATA):
	cargo build
	./target/debug/bootable > $(ISO_DATA)

$(ISO_FILE): $(STAGE1_BIN) $(ISO_DATA)
	# create the 19 sectors required
	dd if=/dev/zero of=$(ISO_FILE) count=20 bs=2048
	# copy iso data in sector 17 and 18
	dd if=$(ISO_DATA) of=$(ISO_FILE) seek=16 count=3 bs=2048 conv=notrunc
	# copy stage 1
	dd if=$(STAGE1_BIN) of=$(ISO_FILE) seek=$$((19*4)) count=4 bs=512 conv=notrunc

$(STAGE1_BIN):
	nasm $(STAGE1) -o $(STAGE1_BIN)

clean:
	cargo clean -p iso9660
	rm -f $(STAGE1_BIN)
	rm -f $(ISO_FILE)
	rm -f $(ISO_DATA)
