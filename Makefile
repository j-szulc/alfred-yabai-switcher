.PHONY: workflow
workflow:
	cd yabai-get-windows && cargo build --release
	mkdir -p build
	cp yabai-get-windows/target/release/yabai-get-windows ./build/
	cp alfredworkflow/* build/
	cd build && zip -r alfred-yabai-switcher.alfredworkflow *