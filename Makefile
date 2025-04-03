NAME	:= kernel.iso
TARGET   := kernel
CARGO_BIN := target/debug/$(TARGET)

KERNEL_ELF := $(TARGET).elf
KERNEL_BIN := $(TARGET).bin

RUSTFLAGS := -C panic=abort

CARGO_OPTS := -Z build-std=core,compiler_builtins -Z build-std-features=panic_immediate_abort --target=target.json

.PHONY: all clean run

all: $(NAME)

boot.o: boot/boot.asm
	@echo "Compiling boot.asm -> boot.o"
	nasm -f elf32 boot/boot.asm -o boot.o

$(TARGET):
	@echo "Cargo compilation"
	RUSTFLAGS="$(RUSTFLAGS)" cargo build $(CARGO_OPTS)

$(KERNEL_ELF): boot.o $(TARGET)
	@echo "Linking : creating $(KERNEL_ELF)"
	i386-elf-ld -T linker.ls -o $(KERNEL_ELF) boot.o $(CARGO_BIN)

$(KERNEL_BIN): $(KERNEL_ELF)
	@echo "Objcopy : converting $(KERNEL_ELF) -> $(KERNEL_BIN)"
	objcopy -O binary $(KERNEL_ELF) $(KERNEL_BIN)

$(NAME): $(KERNEL_BIN)
	@echo "Creating $(NAME)"
	mkdir -p isodir/boot/grub
	cp $(KERNEL_BIN) isodir/boot/$(KERNEL_BIN)
	cp boot/grub.cfg isodir/boot/grub/grub.cfg
	grub-mkrescue -o $(NAME) isodir
	@echo "ISO image $(NAME) created"
	@echo "You can run it with QEMU: make run"

run: all
	@echo "Starting with QEMU..."
	qemu-system-i386 -kernel $(KERNEL_BIN)

clean:
	@echo "Cleaning..."
	cargo clean
	rm -f boot.o $(KERNEL_ELF) $(KERNEL_BIN)
