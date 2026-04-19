.PHONY: build-python build-wasm

build-python:
	@python3 -m pip show maturin > /dev/null 2>&1 || ( \
		echo "maturin not found, installing..."; \
		python3 -m pip install maturin || exit 1; \
	)
	@echo "Building Python package..."
	@python3 -m maturin build --release

build-wasm:
	@command -v wasm-pack > /dev/null 2>&1 || ( \
		echo "wasm-pack not found, installing..."; \
		curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh || exit 1; \
	)
	@echo "Building WASM package..."
	@wasm-pack build --target web
