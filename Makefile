lint:
	black pre apps tests
	isort pre apps tests
	flake8 pre apps tests
	mypy pre apps --disable-error-code import
	pylint pre apps

test:
	pytest --cov=pre --cov-report=term --cov-report=term-missing
	
build_contract:
	bash -c "cd contract; ./compile.sh proxy_reencryption; cd ../"