NAME         := kernel.iso
TARGET       := kernel
BUILD_DIR    := build
TARGET_JSON  := i386-unknown-none.json
BOOT_OBJ     := $(BUILD_DIR)/boot.o
KERNEL_O     := $(BUILD_DIR)/kernel.o
KERNEL_BIN   := $(TARGET).bin
RUST_TOOLCHAIN := +nightly

ifneq ($(shell command -v grub-mkrescue 2>/dev/null),)
  GRUB_MKRESCUE := grub-mkrescue
else ifneq ($(shell command -v i686-elf-grub-mkrescue 2>/dev/null),)
  GRUB_MKRESCUE := i686-elf-grub-mkrescue
else
  $(error "grub-mkrescue not found : install 'grub' or 'i686-elf-grub' using Homebrew")
endif

UNAME := $(shell uname)
ifeq ($(findstring Darwin,$(UNAME)),Darwin)
	LINKER_SCRIPT := boot/linker_macos.ls
	SIZE_CMD := stat -f%z
  ifneq ($(shell command -v i386-elf-ld 2>/dev/null),)
    LD := i386-elf-ld
  else ifneq ($(shell command -v ld.lld 2>/dev/null),)
    LD := ld.lld
  else ifneq ($(shell command -v lld 2>/dev/null),)
    LD := lld
  else
    $(error No ELF linker found : install “i386-elf-binutils” or “llvm” (brew install llvm) and add it to your PATH)
  endif
else
  LD := ld
  LINKER_SCRIPT := boot/linker.ls
  SIZE_CMD := stat -c%s
endif
SIZE_LIMIT := 10485760

.PHONY: all clean run

all: $(NAME)

$(BOOT_OBJ): boot/boot.asm
	@echo "Compiling boot.asm -> $@"
	@mkdir -p $(BUILD_DIR)
	nasm -f elf32 $< -o $@

$(KERNEL_BIN): $(BOOT_OBJ) src/main.rs Cargo.toml $(TARGET_JSON)
	@echo "Building Rust kernel..."
	cargo $(RUST_TOOLCHAIN) build -Z json-target-spec --target $(TARGET_JSON) --release
	@echo "Extracting .a into $(KERNEL_O)..."
	@cp target/i386-unknown-none/release/lib$(TARGET).a $(KERNEL_O)
	@echo "Linking -> $@ with $(LD)..."
	$(LD) -m elf_i386 -T ${LINKER_SCRIPT} -o $@ $(BOOT_OBJ) $(KERNEL_O)
	@size=$$($(SIZE_CMD) $(KERNEL_BIN)); \
		echo "$(KERNEL_BIN) size: $$size bytes"; \
		if [ $$size -gt $(SIZE_LIMIT) ]; then \
			echo "Error: $(KERNEL_BIN) exceeds 10MB ($$size bytes)"; \
			exit 1; \
		fi

$(NAME): $(KERNEL_BIN)
	@echo "Creating ISO -> $@"
	@rm -rf isodir
	@mkdir -p isodir/boot/grub
	@cp $(KERNEL_BIN) isodir/boot/$(KERNEL_BIN)
	@cp boot/grub.cfg isodir/boot/grub/
	@$(GRUB_MKRESCUE) -o $@ isodir
	@size=$$($(SIZE_CMD) $(NAME)); \
		echo "$(NAME) size: $$size bytes"; \
		if [ $$size -gt $(SIZE_LIMIT) ]; then \
			echo "Error: $(NAME) exceeds 10MB ($$size bytes)"; \
			exit 1; \
		fi

run: all
	@echo "Launching QEMU -> $(NAME)"
	qemu-system-i386 -cdrom $(NAME)

clean:
	@echo "Cleaning build artifacts"
	rm -rf $(BUILD_DIR) $(KERNEL_BIN) isodir
	cargo clean

fclean: clean
	rm -f $(NAME)

docker-build:
	docker build --platform=linux/amd64 -t kfs-builder .
	docker run --rm -v "$(PWD):/kernel" -w /kernel kfs-builder make re

re: fclean all
