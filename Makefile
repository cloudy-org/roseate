.PHONY: build

build:
	cargo build --release

install: install-shortcut
	cp ./target/release/roseate /usr/bin/

install-shortcut:
	cp ./app/assets/roseate.desktop /usr/share/applications/
	cp ./app/assets/icons/original.png /usr/share/pixmaps/roseate.png

	update-desktop-database /usr/share/applications/

uninstall: uninstall-shortcut
	rm /usr/bin/roseate

uninstall-shortcut:
	rm /usr/share/pixmaps/roseate.png
	rm /usr/share/applications/roseate.desktop

	update-desktop-database /usr/share/applications/

# pull-submodules:
# 	git submodule update --init --recursive