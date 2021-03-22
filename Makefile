build:
	docker run --rm \
		-v ${PWD}/../smartcrop.rs:/smartcrop.rs \
		-v ${PWD}:/code \
		-v ${HOME}/.cargo/registry:/root/.cargo/registry \
		-v ${HOME}/.cargo/git:/root/.cargo/git \
		softprops/lambda-rust

rust_version:
	rustup override add 1.32.0
