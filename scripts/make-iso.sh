#!/bin/sh
set -eu

kernel=$1
config=$2
iso=$3
limine_bin=$(command -v limine)
limine_dir=$(dirname "$(dirname "$limine_bin")")/share/limine
root=$(mktemp -d)
trap 'rm -rf "$root"' EXIT

mkdir -p "$root/boot"
cp "$kernel" "$root/boot/kernel"
cp "$config" "$root/boot/limine.conf"
cp "$limine_dir/limine-bios.sys" \
    "$limine_dir/limine-bios-cd.bin" \
    "$root/boot/"

xorriso -as mkisofs \
    -b boot/limine-bios-cd.bin \
    -no-emul-boot -boot-load-size 4 -boot-info-table \
    "$root" -o "$iso"
