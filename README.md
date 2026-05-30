# kernel-from-scratch

A minimal i386 kernel written in Rust (with an assembly boot stub), booted via
GRUB and run under QEMU.

## Build

The kernel is built inside a container, so the host only needs a container
engine — no Rust toolchain, linker or GRUB tools required on the host.

### Containerized build (recommended)

Podman is used by default:

```bash
make container-build          # builds inside the container with Podman
```

You can force a specific engine:

```bash
make CONTAINER=podman container-build   # explicit Podman
make CONTAINER=docker container-build   # use Docker instead
```

Aliases are also available for convenience:

```bash
make podman-build             # same as the default container-build
make docker-build             # build with Docker
```

The volume is mounted with the `:z` suffix so it works out of the box on
SELinux-enabled hosts such as Fedora.

#### Fedora host setup

```bash
sudo dnf install podman
```

To run the resulting ISO under QEMU (outside the container):

```bash
sudo dnf install qemu-system-x86
make run
```

### Native build

A native build requires a nightly Rust toolchain, an ELF linker, NASM, GRUB
tools (`grub2-tools` / `grub-mkrescue`) and `xorriso`. Once installed:

```bash
make            # build kernel.iso
make run        # build and launch in QEMU
make re         # clean rebuild
make clean      # remove build artifacts
make fclean     # also remove the ISO
```
