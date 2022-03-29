OVMF := /usr/share/ovmf/OVMF.fd
ESP := esp
BUILD_ARGS :=
QEMU_ARGS := -net none
MODE ?= release

ifeq (${MODE}, release)
	BUILD_ARGS += --release
endif

.PHONY: build run debug clean launch \
	target/x86_64-unknown-uefi/$(MODE)/ggos_boot.efi \
	target/x86_64-unknown-none/$(MODE)/ggos_kernel

run: build launch

launch:
	@qemu-system-x86_64 \
		-bios ${OVMF} \
		-drive format=raw,file=fat:rw:${ESP} \
		$(QEMU_ARGS)

debug: build
	@qemu-system-x86_64 \
		-bios ${OVMF} \
		-drive format=raw,file=fat:rw:${ESP} \
		-s -S

clean:
	@cargo clean

build: $(ESP)

$(ESP): $(ESP)/EFI/BOOT/BOOTX64.EFI $(ESP)/KERNEL.ELF $(ESP)/EFI/BOOT/boot.conf

$(ESP)/EFI/BOOT/BOOTX64.EFI: target/x86_64-unknown-uefi/$(MODE)/ggos_boot.efi
	@mkdir -p $(@D)
	cp $< $@
$(ESP)/EFI/BOOT/boot.conf: pkg/kernel/config/boot.conf
	@mkdir -p $(@D)
	cp $< $@
$(ESP)/KERNEL.ELF: target/x86_64-unknown-none/$(MODE)/ggos_kernel
	@mkdir -p $(@D)
	cp $< $@

target/x86_64-unknown-uefi/$(MODE)/ggos_boot.efi: pkg/boot
	cd pkg/boot && cargo build $(BUILD_ARGS)
target/x86_64-unknown-none/$(MODE)/ggos_kernel: pkg/kernel
	cd pkg/kernel && cargo build $(BUILD_ARGS)
