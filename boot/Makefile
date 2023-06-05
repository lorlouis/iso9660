STAGE1 = stage1.asm
STAGE1_BIN = stage1.bin

FLOPPY_IMG = floppy.img

QEMU = qemu-system-i386

run: $(FLOPPY_IMG)
	$(QEMU) -drive format=raw,file=$(FLOPPY_IMG)

$(FLOPPY_IMG): $(STAGE1_BIN)
	# create the floppy image
	dd if=/dev/zero of=$(FLOPPY_IMG) count=2880 bs=512
	# make vfat fs
	mkfs.vfat $(FLOPPY_IMG)
	# copy stage 1
	dd if=$(STAGE1_BIN) of=$(FLOPPY_IMG) count=1 bs=512 conv=notrunc

$(STAGE1_BIN):
	nasm $(STAGE1) -o $(STAGE1_BIN)

clean:
	rm -f $(STAGE1_BIN)
	rm -f $(FLOPPY_IMG)

