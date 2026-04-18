.PHONY: build-python

build-python:
	@python3 -m pip show maturin > /dev/null 2>&1 || ( \
		echo "maturin not found, installing..."; \
		python3 -m pip install maturin || exit 1; \
	)
	@echo "Building Python package..."
	@python3 -m maturin build --release
