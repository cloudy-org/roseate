build:
	cargo tauri build

install-deps:
	npm i

run:
	cargo tauri dev

generate-icon: # Can be used like so: make generate-icon ICON="./assets/icon.ico"
	cargo tauri icon $(ICON)

clean:
	cd src-tauri && cargo clean && cd ..
	rm -r .next
	rm -r node_modules
	rm package-lock.json