import setuptools


with open("README.md", "r") as fh:
    long_description = fh.read()

setuptools.setup(
    name="proxy-reencryption",
    version="0.1.0",
    author="Fetch AI",
    author_email="developer@fetch.ai",
    description="Fetch AI proxy re-encryption service",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/fetchai/pre",
    packages=setuptools.find_packages(),
    classifiers=[
        # Need to fill in
        "Operating System :: OS Independent",
    ],
    python_requires=">=3.6",
    install_requires=[
        "cosmpy",
        "requests",
        "click",
    ],
    tests_require=["tox~=3.20.0"],
)
