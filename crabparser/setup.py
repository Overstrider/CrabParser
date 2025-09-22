from setuptools import setup, find_packages
from setuptools_rust import Binding, RustExtension

setup(
    name="crabparser",
    version="0.1.0",
    rust_extensions=[
        RustExtension(
            "crabparser.crabparser_rust",
            binding=Binding.PyO3,
            path="Cargo.toml",
            debug=False,
        )
    ],
    packages=find_packages(exclude=["src"]),
    zip_safe=False,
    python_requires=">=3.9",
)