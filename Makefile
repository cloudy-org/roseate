ifeq ($(OS),Windows_NT)     # is Windows_NT on XP, 2000, 7, Vista, 10...
    detected_os := Windows
else
    detected_os := $(shell uname)  # same as "uname -s"
endif

build:
	cargo build --release

install:
ifeq ($(detected_os), Windows)
	copy ".\target\release\roseate.exe" "$(USERPROFILE)\.cargo\bin\"
else
	sudo cp ./target/release/roseate /usr/bin/
endif

uninstall:
ifeq ($(detected_os), Windows)
	del "$(USERPROFILE)\.cargo\bin\roseate.exe"
else
	sudo rm -r /usr/bin/roseate
endif

clean:
	cargo clean

pull-submodules:
	git submodule update --init --recursive

update-submodules:
	git submodule update --recursive --remote