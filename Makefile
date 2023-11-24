.PHONY: workflow
workflow:
	mkdir -p build
	cd yabai-get-windows && cargo build --release --target x86_64-apple-darwin
	cd yabai-get-windows && cargo build --release --target aarch64-apple-darwin
	cd yabai-get-windows/target && lipo -create -output ../../build/yabai-get-windows x86_64-apple-darwin/release/yabai-get-windows aarch64-apple-darwin/release/yabai-get-windows
	cp alfredworkflow/* build/
	cd build && zip -r alfred-yabai-switcher.alfredworkflow *