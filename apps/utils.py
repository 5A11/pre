from pathlib import Path

import click


def file_argument_with_rewrite(*args, **kwargs):
    if not args:
        raise ValueError("Argument name not specified!")

    def deco(func):
        def check_rewrite(
            ctx: click.Context, param, value
        ):  # pylint: disable=unused-argument
            rewrite = ctx.params.pop("rewrite")
            if Path(value).exists() and not rewrite:
                click.echo(
                    f"File `{value}` exists, please use --rewrite option to allow file rewrite"
                )
                ctx.exit(1)
            return value

        required = kwargs.pop("required", True)
        func = click.option(
            "--rewrite", is_flag=True, is_eager=True, expose_value=True
        )(func)
        func = click.argument(
            *args,
            type=click.Path(
                file_okay=True, dir_okay=False, writable=True, path_type=Path
            ),
            required=required,
            expose_value=True,
            callback=check_rewrite,
            **kwargs,
        )(func)
        return func

    return deco


file_exists_type = click.Path(
    file_okay=True,
    dir_okay=False,
    readable=True,
    exists=True,
    path_type=Path,
)


def private_key_file_argument(name: str, *args, **kwargs):
    def deco(func):
        func = click.argument(
            name,
            *args,
            type=file_exists_type,
            **kwargs,
        )(func)
        return func

    return deco


def private_key_file_option(name: str, *args, **kwargs):
    def deco(func):
        func = click.option(
            name,
            *args,
            type=file_exists_type,
            **kwargs,
        )(func)
        return func

    return deco
