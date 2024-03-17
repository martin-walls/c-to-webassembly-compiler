#!/usr/bin/env python
import argparse
import os
import subprocess
import sys
from pathlib import Path
from typing import Generator

import yaml

TEST_PROGRAMS_DIR = Path(__file__).parent.parent.resolve() / "02-test-programs"
TESTS_DIR = Path(__file__).parent.parent.resolve() / "tests"

PROJECT_BUILD_PATH = Path(__file__).parent.parent.resolve() / "target" / "debug" / "c_to_wasm_compiler"

COMPILE_OUTPUT_DIR = TESTS_DIR / "build"
# create output dir if it doesn't exist
os.makedirs(COMPILE_OUTPUT_DIR, exist_ok=True)

NODE_RUNTIME_PATH = Path(__file__).parent.parent.resolve() / "runtime" / "run.mjs"

EXIT_SUCCESS = 0
EXIT_TESTS_FAILED = 1


class TestSpec:
    def __init__(self, name: str, source: Path, args: list[str]):
        self.name = name
        self.source = source
        self.args = args


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


class TestFailedException(Exception):
    """ Raised when a test fails. """

    def __init__(self, message, *args):
        self.message = message

    def __str__(self):
        return f"TestFailedException: {self.message}"


def get_test_specs() -> Generator[TestSpec, None, None]:
    test_spec_files = TESTS_DIR.glob("*.yaml")
    for test_spec_file in test_spec_files:
        if test_spec_file.is_file():
            yield read_test_file(test_spec_file)


def get_wasm_output_filepath(name: str) -> Path:
    return COMPILE_OUTPUT_DIR / f"{name}.wasm"


def get_gcc_output_filepath(name: str) -> Path:
    return COMPILE_OUTPUT_DIR / f"{name}.gcc"


def build_project() -> int:
    print("Building project...")

    process_result = subprocess.run(["cargo", "build"])

    return process_result.returncode


def compile_wasm(filepath: Path, name: str, compiler_args: list[str]) -> (str, int):
    print("\tCompiling wasm...")

    compile_env = os.environ.copy()
    compile_env["RUST_LOG"] = "debug"

    output_filepath = get_wasm_output_filepath(name)

    compile_process_result = subprocess.run(
        [PROJECT_BUILD_PATH, filepath, "-o", output_filepath, *compiler_args],
        capture_output=True, env=compile_env, universal_newlines=True
    )

    # cargo run seems to output to stderr instead of stdout
    return compile_process_result.stderr, compile_process_result.returncode


def run_wasm(name: str, args: list[str]) -> (str, str, int):
    print("\tRunning wasm...")

    run_process_result = subprocess.run(
        [NODE_RUNTIME_PATH, get_wasm_output_filepath(name), *[str(a) for a in args]],
        capture_output=True, universal_newlines=True
    )

    return run_process_result.stdout, run_process_result.stderr, run_process_result.returncode


def compile_gcc(filepath: Path, name: str) -> (str, int):
    print("\tCompiling with GCC...")

    output_filepath = get_gcc_output_filepath(name)

    gcc_process_result = subprocess.run(
        ["gcc", filepath, "-o", output_filepath],
        capture_output=True, universal_newlines=True
    )

    return gcc_process_result.stdout, gcc_process_result.returncode


def run_gcc(name: str, args: list[str]) -> (str, int):
    print("\tRunning GCC output...")

    run_process_result = subprocess.run(
        [get_gcc_output_filepath(name), *[str(a) for a in args]],
        capture_output=True, universal_newlines=True
    )

    return run_process_result.stdout, run_process_result.returncode


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

        return TestSpec(
            test_spec["name"],
            test_spec["source"],
            test_spec["args"],
        )


def run_test(test_spec: TestSpec, wasm_compiler_args: list[str]):
    # compile gcc
    gcc_stdout, gcc_exit_code = compile_gcc(test_spec.source, test_spec.name)

    if gcc_exit_code != 0:
        print("GCC compiler stdout")
        print(gcc_stdout)
        raise TestFailedException("Failed to compile with GCC.")

    # run gcc
    gcc_run_stdout, gcc_run_exit_code = run_gcc(test_spec.name, test_spec.args)

    # compile wasm
    compiler_stdout, compiler_exit_code = compile_wasm(test_spec.source, test_spec.name, wasm_compiler_args)

    if compiler_exit_code != 0:
        print("Wasm compiler stdout:")
        print(compiler_stdout)
        raise TestFailedException("Failed to compile wasm.")

    # run wasm
    wasm_run_stdout, wasm_run_stderr, wasm_run_exit_code = run_wasm(test_spec.name, test_spec.args)

    # compare gcc and wasm
    if (gcc_run_exit_code != wasm_run_exit_code) or (gcc_run_stdout != wasm_run_stdout):
        print("Wasm compiler output:")
        print(compiler_stdout)
        print(f"GCC stdout, with exit code {gcc_run_exit_code}:")
        print(gcc_run_stdout)
        print(f"Wasm stdout, with exit code {wasm_run_exit_code}:")
        print(wasm_run_stdout)
        print("Wasm stderr:")
        print(wasm_run_stderr)
        raise TestFailedException("GCC and wasm outputs didn't match.")


def run_all_tests(test_name_filter: str or None, wasm_compiler_args: list[str]) -> bool:
    # compile rust project
    build_exit_code = build_project()
    if build_exit_code != 0:
        print("Error building project.")
        return

    passed_tests = []
    failed_tests = []

    for test_spec in get_test_specs():
        if test_name_filter is None or test_name_filter in test_spec.name:
            try:
                print(f"Running test: {test_spec.name}")
                run_test(test_spec, wasm_compiler_args)
                print("\tTest passed")
                passed_tests.append(test_spec)
            except TestFailedException as e:
                print(f"\tTest failed: {e.message}")
                failed_tests.append(test_spec)

    print()
    if len(failed_tests) == 0:
        print("All tests passed")
        return True
    else:
        print("Passed tests:")
        for test in passed_tests:
            print(f"\t{test.name}")

        print("Failed tests:")
        for test in failed_tests:
            print(f"\t{test.name}")

        return False


def run_program(test_name_filter: str or None, program_args: list[str], wasm_compiler_args: list[str]):
    # compile rust project
    build_exit_code = build_project()
    if build_exit_code != 0:
        print("Error building project.")
        return

    for test_spec in get_test_specs():
        if test_name_filter is None or test_name_filter in test_spec.name:
            print(f"Running {test_spec.name}")
            # compile wasm
            compiler_stdout, compiler_exit_code = compile_wasm(test_spec.source, test_spec.name, wasm_compiler_args)

            if compiler_exit_code != 0:
                print("Wasm compiler stdout:")
                print(compiler_stdout)
                return

            # run wasm
            # pass supplied args if any, else use the args from the test spec
            wasm_run_stdout, wasm_run_stderr, wasm_run_exit_code = run_wasm(test_spec.name,
                                                                            program_args if len(
                                                                                program_args) > 0 else test_spec.args)

            print("Stdout:")
            print(wasm_run_stdout)
            if wasm_run_stderr:
                print("Stderr:")
                print(wasm_run_stderr)


if __name__ == "__main__":
    # parse CLI args
    parser = argparse.ArgumentParser()
    parser.add_argument("filter", nargs="?", default=None)

    parser.add_argument("--run", "-r", action="store_true",
                        help="run a test file and show output, optionally specifying different arguments")
    # arguments to pass through to the test when using --run
    parser.add_argument("--args", nargs="+", help="arguments to pass through to the test program when using --run")
    # flags to pass to my wasm compiler
    parser.add_argument("--flags", nargs="+", help="Flags to pass to the Wasm compiler")

    args = parser.parse_args()

    wasm_compiler_args = [f"--{flag}" for flag in (args.flags if args.flags else [])]

    if args.run:
        run_program(args.filter, args.args if args.args else [], wasm_compiler_args)
    else:
        all_passed = run_all_tests(args.filter, wasm_compiler_args)
        sys.exit(EXIT_SUCCESS if all_passed else EXIT_TESTS_FAILED)
