NAME         := kernel.iso
TARGET       := kernel
BUILD_DIR    := build
TARGET_JSON  := i386-unknown-none.json
BOOT_OBJ     := $(BUILD_DIR)/boot.o
KERNEL_BIN   := $(TARGET).bin
KERNEL_O     := $(BUILD_DIR)/kernel.o
RUST_TOOLCHAIN := +nightly

ifneq ($(shell command -v grub-mkrescue 2>/dev/null),)
  GRUB_MKRESCUE := grub-mkrescue
else ifneq ($(shell command -v i686-elf-grub-mkrescue 2>/dev/null),)
  GRUB_MKRESCUE := i686-elf-grub-mkrescue
else
  $(error "Aucun grub-mkrescue trouvé : installe 'grub' ou 'i686-elf-grub' via Homebrew")
endif

UNAME := $(shell uname)
ifeq ($(findstring Darwin,$(UNAME)),Darwin)
  ifneq ($(shell command -v i386-elf-ld 2>/dev/null),)
    LD := i386-elf-ld
  else ifneq ($(shell command -v ld.lld 2>/dev/null),)
    LD := ld.lld
  else ifneq ($(shell command -v lld 2>/dev/null),)
    LD := lld
  else
    $(error Aucun linker ELF trouvé : installe “i386-elf-binutils” ou “llvm” (brew install llvm) et ajoute au PATH)
  endif
else
  LD := ld
endif

.PHONY: all clean run

all: $(NAME)

$(BOOT_OBJ): boot/boot.asm
	@echo "Compiling boot.asm -> $@"
	@mkdir -p $(BUILD_DIR)
	nasm -f elf32 $< -o $@

$(KERNEL_BIN): $(BOOT_OBJ) src/main.rs Cargo.toml $(TARGET_JSON)
	@echo "Building Rust kernel..."
	cargo $(RUST_TOOLCHAIN) build --target $(TARGET_JSON) --release
	@echo "Extracting .a into $(KERNEL_O)..."
	@cp target/i386-unknown-none/release/lib$(TARGET).a $(KERNEL_O)
	@echo "Linking -> $@ with $(LD)..."
	$(LD) -m elf_i386 -T boot/linker.ls -o $@ $(BOOT_OBJ) $(KERNEL_O)

$(NAME): $(KERNEL_BIN)
	@echo "Creating ISO -> $@"
	@rm -rf isodir
	@mkdir -p isodir/boot/grub
	@cp $(KERNEL_BIN) isodir/boot/$(KERNEL_BIN)
	@cp boot/grub.cfg isodir/boot/grub/
	@$(GRUB_MKRESCUE) -o $@ isodir

run: all
	@echo "Launching QEMU -> $(NAME)"
	qemu-system-i386 -cdrom $(NAME)

clean:
	@echo "Cleaning build artifacts"
	rm -rf $(BUILD_DIR) $(KERNEL_BIN) isodir
	cargo clean

fclean: clean
	rm -f $(NAME)

re: fclean all
