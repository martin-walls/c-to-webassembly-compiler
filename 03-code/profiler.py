#!/usr/bin/env python
from pathlib import Path
import sys
import argparse
import matplotlib.pyplot as plt
import numpy as np

LOGS_DIR = Path(__file__).parent.resolve() / "logs"

EXIT_CODE_INVALID_ARGS = 1
EXIT_CODE_INVALID_LOG_FILE = 2

# plot config
plt.rc("font", family="serif")
plt.style.use("seaborn-muted")


def read_stack_ptr_log_file(filepath: Path):
    with open(filepath, "r") as log:
        values = []
        line = log.readline()
        while line:
            try:
                values.append(int(line))
            except ValueError:
                print("Invalid stack ptr log file: expected an integer on every line.")
                sys.exit(EXIT_CODE_INVALID_LOG_FILE)
            line = log.readline()

        return values


def plot_stack_memory_usage(stack_ptr_log_file: Path, plot_output_file: Path | None):
    log_values = read_stack_ptr_log_file(stack_ptr_log_file)
    x = np.arange(len(log_values))

    fig, ax = plt.subplots()
    ax.bar(x, log_values, width=1)
    ax.set_xlabel(r"Program execution $\rightarrow$")
    ax.set_xticklabels([])
    ax.set_xticks([])
    ax.set_ylabel("Stack size (bytes)")

    if plot_output_file is not None:
        plt.savefig(plot_output_file)

    plt.show()


def stack_memory_usage_profiler(args):
    stack_ptr_log_file = Path(args.logfile).resolve()

    output_plot_path = args.output
    if output_plot_path is not None:
        output_plot_path = Path(output_plot_path).resolve()

    plot_stack_memory_usage(stack_ptr_log_file, output_plot_path)


if __name__ == "__main__":
    # parse CLI args
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(required=True)

    parser_stack = subparsers.add_parser("stack", help="Stack memory usage profiling")
    parser_stack.add_argument("logfile", help="Path to stack pointer log file")
    parser_stack.add_argument("--output", "-o", help="Path to save plot as PGF file")
    # define the function to call if the stack subcommand is used
    parser_stack.set_defaults(func=stack_memory_usage_profiler)

    args = parser.parse_args()
    args.func(args)
