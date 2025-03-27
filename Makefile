TARGET   := kernel
CARGO_BIN := target/debug/$(TARGET)

KERNEL_ELF := $(TARGET).elf
KERNEL_BIN := $(TARGET).bin

RUSTFLAGS := -C stack-protector=0 -C no-default-libs -C panic=abort

CARGO_OPTS := -Z build-std=core,compiler_builtins

.PHONY: all clean run

all: $(KERNEL_BIN)

boot.o: boot/boot.asm
	@echo "Compiling boot.asm -> boot.o"
	nasm -f elf32 boot/boot.asm -o boot.o

$(TARGET):
	@echo "Cargo compilation"
	RUSTFLAGS="$(RUSTFLAGS)" cargo build $(CARGO_OPTS)

$(KERNEL_ELF): boot.o $(TARGET)
	@echo "Linking : creating $(KERNEL_ELF)"
	ld -m elf_i386 -T linker.ls -o $(KERNEL_ELF) boot.o $(CARGO_BIN)

$(KERNEL_BIN): $(KERNEL_ELF)
	@echo "Objcopy : converting $(KERNEL_ELF) -> $(KERNEL_BIN)"
	objcopy -O binary $(KERNEL_ELF) $(KERNEL_BIN)

run: all
	@echo "Starting with QEMU..."
	qemu-system-i386 -kernel $(KERNEL_BIN)

clean:
	@echo "Cleaning..."
	cargo clean
	rm -f boot.o $(KERNEL_ELF) $(KERNEL_BIN)
