lint:
	black pre apps
	isort pre apps
	flake8 pre apps
	mypy pre apps --disable-error-code import
	pylint pre apps
