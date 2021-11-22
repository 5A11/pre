lint:
	black pre
	isort pre
	flake8 pre
	mypy pre --disable-error-code import
	pylint pre
