.PHONY: workflow copy-binary
workflow:
	mkdir -p build
	cd yabai-get-windows && cargo build --release --target x86_64-apple-darwin
	cd yabai-get-windows && cargo build --release --target aarch64-apple-darwin
	cd yabai-get-windows/target && lipo -create -output ../../build/yabai-get-windows x86_64-apple-darwin/release/yabai-get-windows aarch64-apple-darwin/release/yabai-get-windows
	cp alfredworkflow/* build/
	cd build && zip -r alfred-yabai-switcher.alfredworkflow *

.PHONY: copy-binary
copy-binary:
	@if [ -z "$$DEST" ]; then \
		echo "Usage: make copy-binary DEST=path_to_destination"; \
		exit 1; \
	fi
	cd yabai-get-windows && cargo build
	cp yabai-get-windows/target/debug/yabai-get-windows $$DEST/