## Run the project

To run the project you will need `Docker` and `docker-compose` installed in your system.

On Windows:
- `docker-compose build`
- `docker-compose up`

On other platforms:
- `make build`
- `make run`

To create a superuser account:
- `make manage CMD="createsuperuser"`
