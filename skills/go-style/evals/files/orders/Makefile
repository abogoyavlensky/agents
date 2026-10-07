.PHONY: check test lint

check: lint test

lint:
	gofmt -l . | (! grep .)
	go vet ./...

test:
	go test -race ./...
