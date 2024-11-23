ifeq ($(OS),Windows_NT)     # is Windows_NT on XP, 2000, 7, Vista, 10...
    detected_os := Windows
else
    detected_os := $(shell uname)  # same as "uname -s"
endif

build:
	cargo build --release

install: install-shortcut
ifeq ($(detected_os), Windows)
	copy ".\target\release\roseate.exe" "$(USERPROFILE)\.cargo\bin\"
else
	sudo cp ./target/release/roseate /usr/bin/
endif

install-shortcut:
ifeq ($(detected_os), Windows)
	echo "Not implemented!"
else
	sudo cp ./assets/rose_emojis/google_noto.png /usr/share/icons/roseate.png
	sudo cp ./assets/roseate.desktop /usr/share/applications/
	sudo update-desktop-database /usr/share/applications/
endif

uninstall: uninstall-shortcut
ifeq ($(detected_os), Windows)
	del "$(USERPROFILE)\.cargo\bin\roseate.exe"
else
	sudo rm /usr/bin/roseate
endif

uninstall-shortcut:
ifeq ($(detected_os), Windows)
	echo "Not implemented!"
else
	sudo rm /usr/share/icons/roseate.png
	sudo rm /usr/share/applications/roseate.desktop
	sudo update-desktop-database /usr/share/applications/
endif

clean:
	cargo clean

pull-submodules:
	git submodule update --init --recursive

update-submodules:
	git submodule update --recursive --remote
