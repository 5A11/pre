build:
	docker-compose build

run:
	docker-compose up

clean:
	docker-compose down

manage:
	docker-compose run backend python manage.py $(CMD)

test:
	docker-compose run backend python manage.py test

flake:
	docker-compose run backend flake8 backend

black:
	docker-compose run backend black backend

mypy:
	docker-compose run backend mypy backend

check:
	docker-compose run backend bash -c "black backend && flake8 backend && mypy backend"

loaddata:
	docker-compose run backend python manage.py loaddata \
		pre_backend/fixtures/data_accesses.json \
		pre_backend/fixtures/user_profiles.json \
		pre_backend/fixtures/users.json \
