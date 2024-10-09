ifeq ($(OS),Windows_NT)     # is Windows_NT on XP, 2000, 7, Vista, 10...
    detected_os := Windows
else
    detected_os := $(shell uname)  # same as "uname -s"
endif

build:
	cargo build --release

install:
ifeq ($(detected_os), Windows)
	copy ".\src-tauri\target\release\roseate.exe" "$(USERPROFILE)\.cargo\bin\"
else
	sudo cp ./src-tauri/target/release/roseate /usr/bin/
endif

clean:
	cargo clean

pull-submodules:
	git submodule update --init --recursive

update-submodules:
	git submodule update --recursive --remote