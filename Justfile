set quiet
set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

image_name := "template-x86_64"
qemu_flags := env_var_or_default("QEMUFLAGS", "-m 2G")

default:
    just iso

iso: limine kernel
    rm -rf iso_root
    mkdir -p iso_root/boot iso_root/boot/limine iso_root/EFI/BOOT
    cp -v kernel/kernel iso_root/boot/
    cp -v limine.conf iso_root/boot/limine/
    cp -v limine/limine-uefi-cd.bin iso_root/boot/limine/
    cp -v limine/BOOTX64.EFI iso_root/EFI/BOOT/
    cp -v limine/BOOTIA32.EFI iso_root/EFI/BOOT/
    xorriso -as mkisofs \
        --efi-boot boot/limine/limine-uefi-cd.bin \
        -efi-boot-part --efi-boot-image --protective-msdos-label \
        iso_root -o {{image_name}}.iso
    rm -rf iso_root

hdd: limine kernel
    rm -f {{image_name}}.hdd
    dd if=/dev/zero bs=1M count=0 seek=64 of={{image_name}}.hdd
    sgdisk {{image_name}}.hdd -n 1:2048 -t 1:ef00
    mformat -i {{image_name}}.hdd@@1M
    mmd -i {{image_name}}.hdd@@1M ::/EFI ::/EFI/BOOT ::/boot ::/boot/limine
    mcopy -i {{image_name}}.hdd@@1M kernel/kernel ::/boot
    mcopy -i {{image_name}}.hdd@@1M limine.conf ::/boot/limine
    mcopy -i {{image_name}}.hdd@@1M limine/BOOTX64.EFI ::/EFI/BOOT
    mcopy -i {{image_name}}.hdd@@1M limine/BOOTIA32.EFI ::/EFI/BOOT

run: edk2-ovmf iso
    qemu-system-x86_64 \
        -M q35 \
        -drive if=pflash,unit=0,format=raw,file=edk2-ovmf/ovmf-code-x86_64.fd,readonly=on \
        -cdrom {{image_name}}.iso \
        -serial stdio \
        {{qemu_flags}}

run-hdd: edk2-ovmf hdd
    qemu-system-x86_64 \
        -M q35 \
        -drive if=pflash,unit=0,format=raw,file=edk2-ovmf/ovmf-code-x86_64.fd,readonly=on \
        -hda {{image_name}}.hdd \
        -serial stdio \
        {{qemu_flags}}

build: kernel

debug:
    just --justfile kernel/Justfile debug

check:
    just --justfile kernel/Justfile check

clippy:
    just --justfile kernel/Justfile clippy

fmt:
    just --justfile kernel/Justfile fmt

kernel:
    just --justfile kernel/Justfile

limine:
    test -d limine || git clone https://github.com/limine-bootloader/limine.git --branch=v10.x-binary --depth=1

edk2-ovmf:
    test -d edk2-ovmf || { curl -kLf --retry 3 -o edk2-ovmf-bins.tar.gz https://github.com/osdev0/edk2-ovmf-nightly-bins/releases/latest/download/edk2-ovmf-bins.tar.gz; tar -xzf edk2-ovmf-bins.tar.gz; rm edk2-ovmf-bins.tar.gz; mv edk2-ovmf-bins edk2-ovmf; }

clean:
    just --justfile kernel/Justfile clean
    rm -rf iso_root {{image_name}}.iso {{image_name}}.hdd

distclean: clean
    rm -rf limine edk2-ovmf
