from pathlib import Path

import click
import yaml

from pre.common import AbstractConfig


def file_argument_with_rewrite(*args, **kwargs):
    if not args:
        raise ValueError("Argument name not specified!")

    def deco(func):
        def check_rewrite(ctx: click.Context, param, value):
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


def config_option(option_name: str, config_class: AbstractConfig, **options):
    def _load_and_check_config(ctx: click.Context, param, value):
        del ctx
        del param
        if not value:
            return config_class.make_default()
        data = yaml.safe_load(Path(value).read_text())
        return config_class.validate(data)

    def deco(func):
        func = click.option(
            option_name,
            expose_value=True,
            callback=_load_and_check_config,
            type=click.Path(
                exists=True,
                file_okay=True,
                dir_okay=False,
                readable=True,
                path_type=Path,
            ),
            **options,
        )(func)

        return func

    return deco
