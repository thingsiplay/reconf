
SHELL = /bin/bash

# Will compress distribution binary.
# 0=disable
# 1=enable
COMPRESS_DIST_BIN:=0

# Create and copy the debug build into distribution folder.
# 0=disable
# 1=enable
BUILD_DEBUG:=0

APP_NAME=$(shell grep -E 'name\s*=' Cargo.toml | grep -o '".*"' | tr -d '"')
APP_DEBUG_NAME=$(APP_NAME)d
APP_VERSION=$(shell grep -E 'version\s*=' Cargo.toml | grep -o '".*"' | tr -d '"')

PACKAGE_DEST_DIR:=./$(APP_NAME)
PACKAGE_BASE_FILE:=$(PACKAGE_DEST_DIR)/$(APP_NAME)_$(APP_VERSION)

UPX_FLAGS:=--best
TEST_FLAGS:=
CLIPPY_FLAGS:=-D warnings
CLIPPY_PEDANTIC_FLAGS:=-W clippy::pedantic
BUILD_RELEASE_FLAGS:=
BUILD_DEBUG_FLAGS:=
DOC_FLAGS:=
EXE:=

.DEFAULT_GOAL:=default

default : check test clippy pedantic debug release

all : default linux-gnu linux-musl windows-gnu doc

check :
	cargo fmt -- --check

test :
	cargo test -- $(TEST_FLAGS)

clippy :
	cargo clippy -- $(CLIPPY_FLAGS)

pedantic :
	cargo clippy -- $(CLIPPY_PEDANTIC_FLAGS)

debug :
	if [ $(BUILD_DEBUG) -eq 1 ]; then \
		cargo build $(BUILD_DEBUG_FLAGS); \
	fi

release :
	cargo build --release $(BUILD_RELEASE_FLAGS)
	mkdir -p "$(PACKAGE_DEST_DIR)"
	cp "./target/release/$(APP_NAME)$(EXE)" "$(PACKAGE_DEST_DIR)/"

linux-gnu : TARGET := x86_64-unknown-linux-gnu
linux-gnu : EXE :=
linux-gnu : readme
	$(release-target)
	$(dist-target)

linux-musl : TARGET := x86_64-unknown-linux-musl
linux-musl : EXE :=
linux-musl : readme
	$(release-target)
	$(dist-target)

windows-gnu : TARGET := x86_64-pc-windows-gnu
windows-gnu : EXE := .exe
windows-gnu : readme
	$(release-target)
	$(dist-target)

define release-target =
cargo build --release --target $(TARGET) $(BUILD_RELEASE_FLAGS)
endef

define dist-target =
mkdir -p "$(PACKAGE_DEST_DIR)/$(TARGET)"
cp "./target/$(TARGET)/release/$(APP_NAME)$(EXE)" "$(PACKAGE_DEST_DIR)/$(TARGET)/"
cp "./LICENSE" "$(PACKAGE_DEST_DIR)/$(TARGET)"
cp "./"$(PACKAGE_DEST_DIR)"/README.html" "$(PACKAGE_DEST_DIR)/$(TARGET)"
if [ $(COMPRESS_DIST_BIN) -eq 1 ]; then \
	upx $(UPX_FLAGS) "$(PACKAGE_DEST_DIR)/$(TARGET)/$(APP_NAME)$(EXE)"; \
fi

tar -cf "$(PACKAGE_BASE_FILE)-$(TARGET).tar.gz" "$(PACKAGE_DEST_DIR)/$(TARGET)/"*

if [ $(BUILD_DEBUG) -eq 1 ]; then \
	cp "./target/debug/$(APP_NAME)$(EXE)" "$(PACKAGE_DEST_DIR)/$(APP_DEBUG_NAME)"; \
	gzip -f "$(PACKAGE_DEST_DIR)/$(TARGET)/$(APP_DEBUG_NAME)$(EXE)"; \
fi
endef

readme :
	mkdir -p "$(PACKAGE_DEST_DIR)"
	pandoc "./README.md" -o "$(PACKAGE_DEST_DIR)/README.html"

doc :
	cargo doc $(DOC_FLAGS)

clean :
	cargo clean

# distclean-target :
# 	-cd "$(PACKAGE_DEST_DIR)/$(TARGET)" && rm -f *
# 	-rm -d -f "$(PACKAGE_DEST_DIR)/$(TARGET)"

