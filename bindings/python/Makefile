.PHONY: develop test clean build install

develop:
	maturin develop

test: develop
	pytest tests/

build:
	maturin build --release

install: build
	pip install target/wheels/*.whl

clean:
	rm -rf target/
	rm -rf *.egg-info
	find . -type d -name __pycache__ -exec rm -rf {} +