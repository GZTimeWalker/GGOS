OVMF := tools/OVMF.fd
ESP := esp
BUILD_ARGS :=
QEMU_ARGS := -m 64M
QEMU_OUTPUT := -serial stdio
MODE ?= release
RUN_MODE ?=
CUR_PATH := $(shell pwd)
APP_PATH := $(CUR_PATH)/pkg/app
DBG_INFO := false

APPS := $(shell find $(APP_PATH) -maxdepth 1 -type d)
APPS := $(filter-out $(APP_PATH),$(patsubst $(APP_PATH)/%, %, $(APPS)))
APPS := $(filter-out config,$(APPS))
APPS := $(filter-out .cargo,$(APPS))

ifeq (${MODE}, release)
	BUILD_ARGS := --release
endif

ifeq (${RUN_MODE}, nographic)
	QEMU_OUTPUT = -nographic
endif

# Only add debug info for kernel
# this is required for VSCode GUI debugging
ifeq (${DBG_INFO}, true)
	PROFILE = release-with-debug
	PROFILE_ARGS = --profile=release-with-debug
else
	PROFILE = ${MODE}
	PROFILE_ARGS = $(BUILD_ARGS)
endif


.PHONY: build run debug clean launch intdbg \
	target/x86_64-unknown-uefi/$(MODE)/ggos_boot.efi \
	target/x86_64-unknown-none/$(PROFILE)/ggos_kernel \
	target/x86_64-unknown-ggos/$(MODE)

run: build launch

launch:
	@qemu-system-x86_64 \
		-bios ${OVMF} \
		-net none \
		$(QEMU_ARGS) \
		$(QEMU_OUTPUT) \
		-drive format=raw,file=fat:rw:${ESP}

intdbg:
	@qemu-system-x86_64 \
		-bios ${OVMF} \
		-net none \
		$(QEMU_ARGS) \
		$(QEMU_OUTPUT) \
		-drive format=raw,file=fat:rw:${ESP} \
		-no-reboot -d int,cpu_reset

debug:
	@qemu-system-x86_64 \
		-bios ${OVMF} \
		-net none \
		$(QEMU_ARGS) \
		$(QEMU_OUTPUT) \
		-drive format=raw,file=fat:rw:${ESP} \
		-s -S

clean:
	@cargo clean

list:
	@for dir in $(APPS); do echo $$dir || exit; done

build: $(ESP)

$(ESP): $(ESP)/EFI/BOOT/BOOTX64.EFI $(ESP)/KERNEL.ELF $(ESP)/EFI/BOOT/boot.conf $(ESP)/APP

$(ESP)/EFI/BOOT/BOOTX64.EFI: target/x86_64-unknown-uefi/$(MODE)/ggos_boot.efi
	@mkdir -p $(@D)
	cp $< $@
$(ESP)/EFI/BOOT/boot.conf: pkg/kernel/config/boot.conf
	@mkdir -p $(@D)
	cp $< $@
$(ESP)/KERNEL.ELF: target/x86_64-unknown-none/$(PROFILE)/ggos_kernel
	@mkdir -p $(@D)
	cp $< $@
$(ESP)/APP: target/x86_64-unknown-ggos/$(MODE)
	@for app in $(APPS); do \
		mkdir -p $(ESP)/APP; \
		cp $</ggos_$$app $(ESP)/APP/$$app; \
	done

target/x86_64-unknown-uefi/$(MODE)/ggos_boot.efi: pkg/boot
	cd pkg/boot && cargo build $(BUILD_ARGS)
target/x86_64-unknown-none/$(PROFILE)/ggos_kernel: pkg/kernel
	cd pkg/kernel && cargo build $(PROFILE_ARGS)
target/x86_64-unknown-ggos/$(MODE):
	@for app in $(APPS); do \
		echo "Building $$app"; \
		cd $(APP_PATH)/$$app && cargo build $(BUILD_ARGS) || exit; \
	done
