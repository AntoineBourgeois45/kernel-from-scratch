NAME         := kernel.iso
TARGET       := kernel
BUILD_DIR    := build
TARGET_JSON  := i386-unknown-none.json
BOOT_OBJ     := $(BUILD_DIR)/boot.o
EXC_OBJ      := $(BUILD_DIR)/exceptions.o
GDT_HELPERS_OBJ := $(BUILD_DIR)/gdt_helpers.o
KERNEL_BIN   := $(TARGET).bin
KERNEL_O     := $(BUILD_DIR)/kernel.o

RUST_TOOLCHAIN	:= +nightly

.PHONY: all clean run

all: $(NAME)

$(BOOT_OBJ): boot/boot.asm
	@echo "Compiling boot.asm -> $@"
	@mkdir -p $(BUILD_DIR)
	nasm -f elf32 $< -o $@

$(EXC_OBJ): src/interrupts/exceptions.asm
	@echo "Assembling exceptions.asm → $@"
	@mkdir -p $(BUILD_DIR)
	nasm -f elf32 $< -o $@

$(GDT_HELPERS_OBJ): src/gdt/gdt_helpers.asm
	@echo "Assembling gdt_helpers.asm → $@"
	@mkdir -p $(BUILD_DIR)
	nasm -f elf32 $< -o $@

$(KERNEL_BIN): $(BOOT_OBJ) $(EXC_OBJ) $(GDT_HELPERS_OBJ) src/main.rs Cargo.toml $(TARGET_JSON)
	cargo $(RUST_TOOLCHAIN) build --target $(TARGET_JSON) --release
	@cp target/i386-unknown-none/release/lib$(TARGET).a $(BUILD_DIR)/kernel.o
	@echo "Linking -> $@"
	ld -m elf_i386 -T boot/linker.ls -o $@ $(BOOT_OBJ) $(KERNEL_O) $(EXC_OBJ) $(GDT_HELPERS_OBJ)

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
	rm -rf $(BUILD_DIR) $(KERNEL_BIN) $(KERNEL_BIN) isodir
	cargo clean

fclean: clean
	rm -rf $(NAME) 

re: fclean all
