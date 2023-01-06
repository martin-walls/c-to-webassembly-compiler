#!/usr/bin/env python
import subprocess
import os
from pathlib import Path
import yaml

TEST_PROGRAMS_DIR = Path(__file__).parent.parent.resolve() / "02-test-programs"
TESTS_DIR = Path(__file__).parent.resolve() / "tests"

PROJECT_BUILD_PATH = Path(__file__).parent.resolve() / "target" / "debug" / "c_to_wasm_compiler"

COMPILE_OUTPUT_DIR = TESTS_DIR / "build"
# create output dir if it doesn't exist
os.makedirs(COMPILE_OUTPUT_DIR, exist_ok=True)

NODE_RUNTIME_PATH = Path(__file__).parent.resolve() / "runtime" / "run.mjs"


def get_wasm_output_filepath(name: str) -> Path:
    return COMPILE_OUTPUT_DIR / f"{name}.wasm"


def get_gcc_output_filepath(name: str) -> Path:
    return COMPILE_OUTPUT_DIR / f"{name}.gcc"


def build_project() -> int:
    print("Building project...")

    process_result = subprocess.run(["cargo", "build"])

    return process_result.returncode


def compile_wasm(filepath: Path, name: str) -> (str, int):
    print("\tCompiling wasm...")

    compile_env = os.environ.copy()
    compile_env["RUST_LOG"] = "debug"

    output_filepath = get_wasm_output_filepath(name)

    # compile_process_result = subprocess.run(
    #     ["cargo", "run", "--", filepath, "-o", output_filepath],
    #     capture_output=True, env=compile_env, universal_newlines=True
    # )

    compile_process_result = subprocess.run(
        [PROJECT_BUILD_PATH, filepath, "-o", output_filepath],
        capture_output=True, env=compile_env, universal_newlines=True
    )

    # cargo run seems to output to stderr instead of stdout
    return compile_process_result.stderr, compile_process_result.returncode


def run_wasm(name: str, args: list[str]) -> (str, str, int):
    print("\tRunning wasm...")

    run_process_result = subprocess.run(
        [NODE_RUNTIME_PATH, get_wasm_output_filepath(name), *args],
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
        [get_gcc_output_filepath(name), *args],
        capture_output=True, universal_newlines=True
    )

    return run_process_result.stdout, run_process_result.returncode


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


def run_test(test_spec: TestSpec):
    # compile gcc
    gcc_stdout, gcc_exit_code = compile_gcc(test_spec.source, test_spec.name)

    if gcc_exit_code != 0:
        print("GCC compiler stdout")
        print(gcc_stdout)
        raise TestFailedException("Failed to compile with GCC.")

    # run gcc
    gcc_run_stdout, gcc_run_exit_code = run_gcc(test_spec.name, test_spec.args)

    # compile wasm
    compiler_stdout, compiler_exit_code = compile_wasm(test_spec.source, test_spec.name)

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
        print(f"Wasm stderr:")
        print(wasm_run_stderr)
        raise TestFailedException("GCC and wasm outputs didn't match.")


def run_all_tests(tests_dir: Path):
    # compile rust project
    build_exit_code = build_project()
    if build_exit_code != 0:
        print("Error building project.")
        exit()

    passed_tests = []
    failed_tests = []

    test_spec_files = tests_dir.glob("*.yaml")
    for test_spec_file in test_spec_files:
        if test_spec_file.is_file():
            test_spec = read_test_file(test_spec_file)

            try:
                print(f"Running test: {test_spec.name}")
                run_test(test_spec)
                print("\tTest passed")
                passed_tests.append(test_spec)
            except TestFailedException as e:
                print(f"\tTest failed: {e.message}")
                failed_tests.append(test_spec)

    print()
    if len(failed_tests) == 0:
        print("All tests passed")
    else:
        print("Passed tests:")
        for test in passed_tests:
            print(f"\t{test.name}")

        print("Failed tests:")
        for test in failed_tests:
            print(f"\t{test.name}")


if __name__ == "__main__":
    run_all_tests(TESTS_DIR)
