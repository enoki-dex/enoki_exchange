SHELL = /bin/zsh

.PHONY: all
all: clean init deps install test

.PHONY: start
start: init deps install

.PHONY: init
.SILENT: init
init:
	dfx start --background

.PHONY: deps
.SILENT: deps
deps:
	./scripts/install_dependencies.sh

.PHONY: install
.SILENT: install
install:
	./scripts/install.sh

.PHONY: init-local
.SILENT: init-local
init-local:
	./scripts/initalize_local_balance.sh $(II_PRINCIPAL)

.PHONE: deploy
.SILENT: deploy
deploy:
	./scripts/deploy.sh

.PHONY: build
.SILENT: build
build:
	dfx canister create --all
	dfx build

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
