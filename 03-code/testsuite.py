#!/usr/bin/env python
import subprocess
import os
from pathlib import Path
import yaml

TEST_PROGRAMS_DIR = Path(__file__).parent.parent.resolve() / "02-test-programs"
TESTS_DIR = Path(__file__).parent.resolve() / "tests"

COMPILE_OUTPUT_DIR = TESTS_DIR / "build"
# create output dir if it doesn't exist
os.makedirs(COMPILE_OUTPUT_DIR, exist_ok=True)

NODE_RUNTIME_PATH = Path(__file__).parent.resolve() / "runtime" / "run.mjs"


def get_compile_output_filepath(name: str) -> Path:
    return COMPILE_OUTPUT_DIR / f"{name}.wasm"


def compile(filepath: Path, name: str) -> str:
    compile_env = os.environ.copy()
    compile_env["RUST_LOG"] = "debug"

    output_filepath = get_compile_output_filepath(name)

    compile_process_result = subprocess.run(
        ["cargo", "run", "--", filepath, '-o', output_filepath],
        capture_output=True, env=compile_env, universal_newlines=True)

    return compile_process_result.stdout


def run_wasm(filepath: Path, args: list[str]) -> (str, int):
    run_process_result = subprocess.run(
        [NODE_RUNTIME_PATH, filepath, *args],
        capture_output=True, universal_newlines=True
    )

    return run_process_result.stdout, run_process_result.returncode


class TestSpec:
    def __init__(self, name: str, source: Path, args: list[str], exit_code: int, stdout: str):
        self.name = name
        self.source = source
        self.args = args
        self.exit_code = exit_code
        self.stdout = stdout


class InvalidTestSpecFileException(Exception):
    """ Raised when the test spec file's syntax is malformed. """

    def __init__(self, *args):
        if args:
            self.message = args[0]
        else:
            self.message = None

    def __str__(self):
        if self.message:
            return f"InvalidTestSpecFileException: {self.message}"
        else:
            return "InvalidTestSpecFileException"


def read_test_file(filepath: Path) -> TestSpec:
    with open(filepath, "r") as f:
        # load spec from file
        test_spec = yaml.safe_load(f)

        # validate file contents
        #
        # name field is required
        if "name" not in test_spec.keys() or test_spec["name"] is None:
            raise InvalidTestSpecFileException("The 'name' field is required")

        # source field is required
        if "source" not in test_spec.keys() or test_spec["source"] is None:
            raise InvalidTestSpecFileException("The 'source' field is required")

        # make source relative to test programs dir
        test_spec["source"] = TEST_PROGRAMS_DIR / test_spec["source"]

        if "args" not in test_spec.keys() or test_spec["args"] is None:
            test_spec["args"] = []

        # exit code field is required
        if "exit code" not in test_spec.keys() or test_spec["exit code"] is None:
            raise InvalidTestSpecFileException("The 'exit code' field is required")

        if "stdout" not in test_spec.keys() or test_spec["stdout"] is None:
            test_spec["stdout"] = ""

        return TestSpec(
            test_spec["name"],
            test_spec["source"],
            test_spec["args"],
            test_spec["exit code"],
            test_spec["stdout"]
        )


def run_tests(tests_dir: Path):
    test_spec_files = tests_dir.glob("*.yaml")
    for test_spec_file in test_spec_files:
        if test_spec_file.is_file():
            test_spec = read_test_file(test_spec_file)
            print(f"Running test: {test_spec.name}")

            print("\tCompiling...")
            compiler_stdout = compile(test_spec.source, test_spec.name)

            print("\tRunning program...")
            run_stdout, exit_code = run_wasm(get_compile_output_filepath(test_spec.name), test_spec.args)

            test_passed = True

            if test_spec.exit_code != exit_code:
                test_passed = False

            if test_spec.stdout != run_stdout:
                test_passed = False

            if test_passed:
                print("\tPassed")
            else:
                print("\tFailed")
                print("Compiler output:")
                print(compiler_stdout)
                print()
                print("Program output:")
                print(run_stdout)
                print()
                print("Expected output:")
                print(test_spec.stdout)
                print()
                print(f"Program exit code: {exit_code}, expected: {test_spec.exit_code}")


if __name__ == "__main__":
    run_tests(TESTS_DIR)
