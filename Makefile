NAME         := kernel.iso
TARGET       := kernel
BUILD_DIR    := build
TARGET_JSON  := i386-unknown-none.json

RUST_SOURCES := $(shell find src -type f -name '*.rs')

BOOT_OBJ       := $(BUILD_DIR)/boot.o
INTERRUPTS_OBJ := $(BUILD_DIR)/interrupts.o
KERNEL_O       := $(BUILD_DIR)/kernel.o
KERNEL_BIN     := $(TARGET).bin

RUST_TOOLCHAIN := +nightly

# Detect grub-mkrescue
ifneq ($(shell command -v grub-mkrescue 2>/dev/null),)
	GRUB_MKRESCUE := grub-mkrescue
else ifneq ($(shell command -v grub2-mkrescue 2>/dev/null),)
	GRUB_MKRESCUE := grub2-mkrescue
else ifneq ($(shell command -v i686-elf-grub-mkrescue 2>/dev/null),)
	GRUB_MKRESCUE := i686-elf-grub-mkrescue
else
	GRUB_MKRESCUE = $(error grub-mkrescue not found: install grub tools or use 'make container-build')
endif

# OS-specific linker configuration
UNAME := $(shell uname)

ifeq ($(findstring Darwin,$(UNAME)),Darwin)

	LINKER_SCRIPT := boot/linker_macos.ls

	ifneq ($(shell command -v i386-elf-ld 2>/dev/null),)
		LD := i386-elf-ld
	else ifneq ($(shell command -v ld.lld 2>/dev/null),)
		LD := ld.lld
	else ifneq ($(shell command -v lld 2>/dev/null),)
		LD := lld
	else
		LD = $(error No ELF linker found: install i386-elf-binutils or LLVM, or use 'make container-build')
	endif

else

	LD := ld
	LINKER_SCRIPT := boot/linker.ls

endif

.PHONY: all clean fclean re run container-build podman-build docker-build

all: $(NAME)

# ---------------------------------------------------------------------------
# Assembly
# ---------------------------------------------------------------------------

$(BOOT_OBJ): boot/boot.asm
	@echo "Compiling boot.asm -> $@"
	@mkdir -p $(BUILD_DIR)
	nasm -f elf32 $< -o $@

$(INTERRUPTS_OBJ): boot/interrupts.asm
	@echo "Compiling interrupts.asm -> $@"
	@mkdir -p $(BUILD_DIR)
	nasm -f elf32 $< -o $@

# ---------------------------------------------------------------------------
# Kernel
# ---------------------------------------------------------------------------

$(KERNEL_BIN): $(BOOT_OBJ) $(INTERRUPTS_OBJ) $(RUST_SOURCES) Cargo.toml $(TARGET_JSON)
	@echo "Building Rust kernel..."
	cargo $(RUST_TOOLCHAIN) -Zjson-target-spec build --target $(TARGET_JSON) --release

	@echo "Extracting Rust static library -> $(KERNEL_O)"
	@cp target/i386-unknown-none/release/lib$(TARGET).a $(KERNEL_O)

	@echo "Linking -> $@ with $(LD)"
	$(LD) -m elf_i386 -T $(LINKER_SCRIPT) -o $@ \
		$(BOOT_OBJ) \
		$(INTERRUPTS_OBJ) \
		$(KERNEL_O)

	@echo "Kernel size:"
	@ls -lh $(KERNEL_BIN)

# ---------------------------------------------------------------------------
# ISO
# ---------------------------------------------------------------------------

$(NAME): $(KERNEL_BIN)
	@echo "Creating ISO -> $@"
	@rm -rf isodir
	@mkdir -p isodir/boot/grub
	@cp $(KERNEL_BIN) isodir/boot/$(KERNEL_BIN)
	@cp boot/grub.cfg isodir/boot/grub/
	$(GRUB_MKRESCUE) -o $@ isodir

	@echo "ISO size:"
	@ls -lh $(NAME)

# ---------------------------------------------------------------------------
# Run
# ---------------------------------------------------------------------------

run: all
	@echo "Launching QEMU -> $(NAME)"
	qemu-system-i386 -cdrom $(NAME)

# ---------------------------------------------------------------------------
# Cleaning
# ---------------------------------------------------------------------------

clean:
	@echo "Cleaning build artifacts..."
	rm -rf $(BUILD_DIR)
	rm -rf isodir
	rm -f $(KERNEL_BIN)
	cargo clean

fclean: clean
	@echo "Removing ISO..."
	rm -f $(NAME)

re: fclean all

# ---------------------------------------------------------------------------
# Container build
# ---------------------------------------------------------------------------

CONTAINER ?= podman

container-build:
	$(CONTAINER) build --platform=linux/amd64 -t kfs-builder .
	$(CONTAINER) run --rm \
		-v "$(PWD):/kernel:z" \
		-w /kernel \
		kfs-builder \
		make re

podman-build: CONTAINER := podman
podman-build: container-build

docker-build: CONTAINER := docker
docker-build: container-build