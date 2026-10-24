.PHONY: setup up down migrate test seed lint build clean logs

setup:
	cp -n .env.example .env 2>/dev/null || true
	go mod download

up:
	docker-compose up -d

down:
	docker-compose down

migrate:
	docker-compose exec -T postgres psql -U eqs -d equalizer_stake -f /docker-entrypoint-initdb.d/001_create_extensions.sql
	docker-compose exec -T postgres psql -U eqs -d equalizer_stake -f /docker-entrypoint-initdb.d/002_create_resources.sql
	docker-compose exec -T postgres psql -U eqs -d equalizer_stake -f /docker-entrypoint-initdb.d/003_create_schedules.sql
	docker-compose exec -T postgres psql -U eqs -d equalizer_stake -f /docker-entrypoint-initdb.d/004_create_reservations.sql
	docker-compose exec -T postgres psql -U eqs -d equalizer_stake -f /docker-entrypoint-initdb.d/005_create_users.sql
	docker-compose exec -T postgres psql -U eqs -d equalizer_stake -f /docker-entrypoint-initdb.d/006_create_analytics.sql

test:
	go test ./tests/... -v -race -count=1

test-unit:
	go test ./tests/unit/... -v -race -count=1

test-integration:
	go test ./tests/integration/... -v -race -count=1

test-concurrency:
	go test ./tests/concurrency/... -v -race -count=1 -timeout=60s

seed:
	go run ./cmd/server/main.go -seed

lint:
	golangci-lint run ./...

build:
	go build -o bin/server ./cmd/server
	go build -o bin/worker ./cmd/worker

clean:
	rm -rf bin/
	rm -rf tmp/

logs:
	docker-compose logs -f
