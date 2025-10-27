.PHONY: develop test clean build install

develop:
	PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 maturin develop

test: develop
	pytest tests/

build:
	PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 maturin build --release

install: build
	pip install target/wheels/*.whl

clean:
	rm -rf target/
	rm -rf *.egg-info
	find . -type d -name __pycache__ -exec rm -rf {} +