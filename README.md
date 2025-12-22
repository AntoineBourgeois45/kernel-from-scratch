KFS Kernel (Rust)

Overview
This repository implements a minimal x86 (i386) kernel in Rust for the 42 "Kernel From Scratch" projects.
It boots via GRUB (Multiboot), provides a VGA text terminal, keyboard input, multiple screens, a tiny shell,
and a KFS-3 memory system with paging + allocators.

What is implemented
KFS-1 (Grub, boot and screen)
- Multiboot ASM entry point and stack setup.
- GRUB bootable ISO build.
- VGA text output + colors, scrolling, cursor control.
- Keyboard input + helpers, multiple screens.

KFS-2 (GDT & Stack)
- Global Descriptor Table at 0x00000800 with kernel/user code/data/stack segments.
- Segment reload after lgdt.
- Stack dump tool (F8 / shell command `stack`).
- Minimal debugging shell with basic commands.

KFS-3 (Memory)
- Paging enabled (CR3 + CR0.PG).
- Page directory + page tables (identity map 0..32 MB).
- Physical frame allocator (bitmap).
- Kernel heap (kmalloc/kfree/ksize/kbrk).
- Virtual heap (vmalloc/vfree/vsize/vbrk).
- Kernel panic handling (print + halt).
- Demonstration commands: meminfo/memtest/memdemo/map/memdump/pmem.

Build requirements
- Rust nightly + rust-src
- nasm
- grub-mkrescue (or i686-elf-grub-mkrescue)
- ld (or lld / i386-elf-ld on macOS)
- qemu-system-i386

Build and run
- Build ISO:
  - `make`
- Run in QEMU:
  - `make run`

Notes
- `make` prints the size of `kernel.bin` and `kernel.iso` and fails if either exceeds 10 MB.
- `make run` uses `-boot d` to boot from the ISO.

Kernel controls
Function keys
- F1: help
- F2: toggle Terminal/Navigation mode
- F3: clear screen
- F4: toggle cursor visibility
- F5/F6/F7: switch screen 1/2/3
- F8: dump kernel stack

Shell commands
- `help` : show help
- `stack` : dump kernel stack
- `memtest` : test paging + allocators
- `meminfo` : show memory stats (paging/PMM/heaps + mappings)
- `memdemo` : alloc/free demo with before/after stats
- `memdump <addr> <len>` : hex dump (max 256 bytes)
- `map <addr>` : show PDE/PTE + flags + phys mapping
- `pmem [start] [count]` : list frame usage
- `clear` / `reboot` / `halt`

Test trame (KFS-3 demo)
Run these in the shell after boot:
1) `meminfo`
2) `memtest`
3) `memdemo`
4) `map 0x000B8000`
5) `map 0x40000000`
6) `memdump 0x000B8000 64`
7) `pmem 0 32`

Memory layout notes
- Identity mapped: 0x00000000 .. 0x01FFFFFF (32 MB).
- Kernel space: <= 0xBFFFFFFF.
- User space: >= 0xC0000000.
- Kernel heap (physical): starts at 0x00400000.
- Virtual heap: starts at 0x40000000.
- PMM assumes 64 MB RAM (adjustable in `src/memory/pmm.rs`).

Project structure (key files)
- `boot/boot.asm` : Multiboot entry and stack setup
- `boot/linker.ls` : linker script
- `src/vga/terminal.rs` : VGA text terminal + logging
- `src/ps2/keyboard.rs` : keyboard scancode parsing
- `src/inputs/handlers.rs` : input handling + help screen
- `src/gdt.rs` : GDT setup (KFS-2)
- `src/stack.rs` : stack dump (KFS-2)
- `src/shell.rs` : kernel shell + commands
- `src/memory/*` : paging + PMM + heaps (KFS-3)
