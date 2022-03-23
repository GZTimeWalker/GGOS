EFI := target/x86_64-unknown-uefi/release/ggos-boot.efi
DEBUG_SYM := target/x86_64-unknown-uefi/debug/ggos-boot.efi
OVMF := /usr/share/ovmf/OVMF.fd
ESP := esp
QEMU_ARGS := -net none

.PHONY: build run header asm doc debug clean launch

build:
	@cargo build --release
	@mkdir -p $(ESP)/EFI/Boot
	@cp $(EFI) $(ESP)/EFI/Boot/BootX64.efi

clippy:
	cargo clippy $(BUILD_ARGS)

doc:
	cargo doc

run: build launch

launch:
	@qemu-system-x86_64 \
		-bios ${OVMF} \
		-drive format=raw,file=fat:rw:${ESP} \
		$(QEMU_ARGS)

debug: build
	@qemu-system-x86_64 -bios ${OVMF} -drive format=raw,file=fat:rw:${ESP} -s -S

header:
	@rust-objdump -h $(EFI) | less

asm:
	@rust-objdump -d $(EFI) | less

clean:
	@cargo clean
