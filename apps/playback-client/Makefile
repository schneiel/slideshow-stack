export PATH := /opt/zig:$(PATH)

TARGET := armv7-unknown-linux-gnueabihf.2.35

build-arm:
	ulimit -n 10240 && \
	CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc \
	cargo zigbuild --target $(TARGET) --features drm

release-arm:
	ulimit -n 10240 && \
	CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc \
	cargo zigbuild --target $(TARGET) --release --features drm

clean-arm:
	rm -rf target/armv7-unknown-linux-gnueabihf