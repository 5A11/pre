lint:
	black pre apps
	isort pre apps
	flake8 pre apps
	mypy pre apps --disable-error-code import
	pylint pre apps

test:
	pytest --cov=pre --cov-report=term --cov-report=term-missing
	
build_contract:
	bash -c "cd contract; ./compile.sh proxy_reencryption; cd ../"