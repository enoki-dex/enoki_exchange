SHELL = /bin/zsh

.PHONY: init
.SILENT: init
init:
	dfx start --background

.PHONY: deps
.SILENT: deps
deps:
	./scripts/install_dependencies.sh

.PHONE: deploy
.SILENT: deploy
deploy:
	./scripts/deploy.sh

.PHONY: build
.SILENT: build
build:
	./scripts/build.sh

.PHONY: test
.SILENT: test
test:
	./tests/test_liquidity_pool_depositing.sh
	./tests/test_exchange.sh
	./tests/test_liquidity_pool_withdrawing.sh

.PHONY: clean
.SILENT: clean
clean:
	dfx stop
	rm -rf .dfx
