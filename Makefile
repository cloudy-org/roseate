BUNDLES = "none"

build:
	cargo tauri build --bundles $(BUNDLES)

install-deps:
	npm i

pull-submodules:
	git submodule update --init --recursive

update-submodules:
	git submodule update --recursive --remote

ARGS = ""

run:
	cargo tauri dev -- -- $(ARGS)

generate-icon: # Can be used like so: make generate-icon ICON="./assets/icon.ico"
	cargo tauri icon $(ICON)

clean:
	cd src-tauri && cargo clean && cd ..
	rm -r .next
	rm -r node_modules
	rm package-lock.json
