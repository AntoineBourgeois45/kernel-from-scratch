NAME         := kernel.iso
TARGET       := kernel
BUILD_DIR    := build
TARGET_JSON  := i386-unknown-none.json
BOOT_OBJ     := $(BUILD_DIR)/boot.o
KERNEL_ELF   := $(TARGET).elf
KERNEL_BIN   := $(TARGET).bin
KERNEL_O     := target/i386-unknown-none/debug/deps/kernel-*.o

RUST_TOOLCHAIN	:= +nightly
RUSTFLAGS     	:= -C panic=abort -C relocation-model=static
BUILD_STD 		:= -Z build-std=core,compiler_builtins -Z build-std-features=compiler-builtins-mem

.PHONY: all clean run

all: $(NAME)

$(BOOT_OBJ): boot/boot.asm
	@echo "Compiling boot.asm -> $@"
	@mkdir -p $(BUILD_DIR)
	nasm -f elf32 $< -o $@

$(KERNEL_ELF): $(BOOT_OBJ) src/main.rs Cargo.toml $(TARGET_JSON)
	@echo "Building kernel ELF -> $@"
	@cargo $(RUST_TOOLCHAIN) rustc \
		--target $(TARGET_JSON) \
		$(BUILD_STD) \
		--profile dev \
		-- --emit=obj \
		$(RUSTFLAGS)

	@echo "Linking kernel ELF -> $@"
	ld -m elf_i386 -T boot/linker.ls $(BOOT_OBJ) $(KERNEL_O) -o $@

$(KERNEL_BIN): $(KERNEL_ELF)
	@echo "Objcopy -> $@"
	objcopy -O binary $< $@

$(NAME): $(KERNEL_BIN)
	@echo "Creating ISO -> $@"
	@mkdir -p isodir/boot/grub
	cp $(KERNEL_BIN) isodir/boot/$(KERNEL_BIN)
	cp boot/grub.cfg isodir/boot/grub/
	grub-mkrescue -o $@ isodir

run: all
	@echo "Launching QEMU -> $(NAME)"
	qemu-system-i386 -cdrom $(NAME)

clean:
	@echo "Cleaning build artifacts"
	rm -rf $(BUILD_DIR) $(KERNEL_ELF) $(KERNEL_BIN) isodir
	cargo clean

re: clean all
