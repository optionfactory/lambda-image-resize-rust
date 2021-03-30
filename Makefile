test:
	cargo test -- --test-threads 1
build:
	docker run --rm \
		-v ${PWD}:/code \
		-v ${HOME}/.cargo/registry:/root/.cargo/registry \
		-v ${HOME}/.cargo/git:/root/.cargo/git \
		softprops/lambda-rust

deps:
	cd .. && git clone https://github.com/softprops/lambda-rust.git # up-to-date docker container might be needed
	cd ../lambda-rust && make build

rust_version:
	rustup override add 1.32.0
