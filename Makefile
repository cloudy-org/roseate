ifeq ($(OS),Windows_NT)     # is Windows_NT on XP, 2000, 7, Vista, 10...
    detected_os := Windows
else
    detected_os := $(shell uname)  # same as "uname -s"
endif

BUNDLES = "none"

build:
	cargo tauri build --bundles $(BUNDLES)

install:
ifeq ($(detected_os), Windows)
	copy ".\src-tauri\target\release\roseate-old.exe" "$(USERPROFILE)\.cargo\bin\"
else
	sudo cp ./src-tauri/target/release/roseate-old /usr/bin/
	sudo mkdir /usr/lib/roseate-old -p
	sudo cp ./src-tauri/target/release/_up_ /usr/lib/roseate-old/ -r
endif

install-deps:
	npm i

uninstall:
ifeq ($(detected_os), Windows)
	del "$(USERPROFILE)\.cargo\bin\roseate-old.exe"
else
	sudo rm /usr/bin/roseate-old
	sudo rm -r /usr/lib/roseate-old/
endif

pull-submodules:
	git submodule update --init --recursive

update-submodules:
	git submodule update --recursive --remote

ARGS = ""

run:
	cargo tauri dev -- -- $(ARGS)

generate-icon: # Can be used like so: make generate-icon ICON="./assets/icon.ico"
	cargo tauri icon $(ICON)

test:
	npx eslint . --ext .ts,.tsx

clean:
	cd src-tauri && cargo clean && cd ..
	rm -r node_modules
	rm package-lock.json
