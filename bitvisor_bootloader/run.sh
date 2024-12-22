#!/bin/sh

mkdir -p bin/EFI/BOOT/
mv $1 bin/EFI/BOOT/BOOTX64.EFI

qemu-system-x86_64 \
    -cpu host -enable-kvm -bios ./OVMF_x86.fd \
    -drive file=fat:rw:bin/,format=raw \
    -nic user,model=e1000e -M q35 -m 4096 \
    --nographic


#    -cpu max -bios ./OVMF_x86.fd \
