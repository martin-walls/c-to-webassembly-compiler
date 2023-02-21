#!/usr/bin/env python
import argparse
import sys
from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np

LOGS_DIR = Path(__file__).parent.resolve() / "logs"

EXIT_CODE_INVALID_ARGS = 1
EXIT_CODE_INVALID_LOG_FILE = 2

# plot config
plt.rc("font", family="serif")
plt.rc("text", usetex=True)
plt.rc("figure", autolayout=True)
# plt.rc("figure", labelsize=12)
plt.rc("font", size=12)
plt.rc("ytick", alignment="center")
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


def plot_max_value(ax, max_value, ymax):
    max_value_colour = "#D65F5F"
    ax.axhline(max_value, linestyle="--", color=max_value_colour)
    ax.text(0, max_value + ymax * 0.025, f"$\\max = {max_value}$", color=max_value_colour)


def plot_stack_memory_usage(stack_ptr_log_file: Path, plot_output_file: Path | None, title: str, show_plot: bool):
    log_values = read_stack_ptr_log_file(stack_ptr_log_file)
    max_value = np.max(log_values)
    x = np.arange(len(log_values))

    fig, ax = plt.subplots()
    ax.bar(x, log_values, width=1, rasterized=True)
    ax.set_xlabel(r"Program execution $\rightarrow$")
    ax.set_xticklabels([])
    ax.set_xticks([])
    ax.set_ylabel("Stack size (bytes)")
    if title:
        ax.set_title(title)

    ymax = max_value * 1.1
    ax.set_ylim([0, ymax])

    plot_max_value(ax, max_value, ymax)

    if plot_output_file is not None:
        plt.savefig(plot_output_file)
        print(f"Plot saved to {plot_output_file}")

    if show_plot:
        plt.show()


def compare_stack_memory_usage(stack_ptr_log_file_1: Path, stack_ptr_log_file_2: Path, plot_output_file: Path | None,
                               title: str, subtitle1: str, subtitle2: str, show_plot: bool):
    log_values_1 = read_stack_ptr_log_file(stack_ptr_log_file_1)
    log_values_2 = read_stack_ptr_log_file(stack_ptr_log_file_2)

    max_value_1 = np.max(log_values_1)
    max_value_2 = np.max(log_values_2)

    fig, (ax1, ax2) = plt.subplots(1, 2, sharey="all", figsize=(8, 4))

    x1 = np.arange(len(log_values_1))
    x2 = np.arange(len(log_values_2))

    ax1.bar(x1, log_values_1, width=1, rasterized=True)
    ax1.set_xlabel(r"Program execution $\rightarrow$")
    ax1.set_xticklabels([])
    ax1.set_xticks([])
    ax1.set_ylabel("Stack size (bytes)")
    if subtitle1:
        ax1.set_title(subtitle1)

    # add some extra space at top of y axis, to allow for "max" text
    ymax = max(max_value_1, max_value_2) * 1.1
    ax1.set_ylim([0, ymax])

    plot_max_value(ax1, max_value_1, ymax)

    ax2.bar(x2, log_values_2, width=1, rasterized=True)
    ax2.set_xlabel(r"Program execution $\rightarrow$")
    ax2.set_xticklabels([])
    ax2.set_xticks([])
    if subtitle2:
        ax2.set_title(subtitle2)

    plot_max_value(ax2, max_value_2, ymax)

    if title:
        fig.suptitle(title)

    if plot_output_file is not None:
        plt.savefig(plot_output_file, dpi=1200)
        print(f"Plot saved to {plot_output_file}")

    if show_plot:
        plt.show()


def stack_memory_usage_profiler(args):
    stack_ptr_log_file = Path(args.logfile).resolve()

    output_plot_path = args.output
    if output_plot_path is not None:
        output_plot_path = Path(output_plot_path).resolve()

    if args.compare:
        stack_ptr_log_file_2 = Path(args.logfile2).resolve()
        compare_stack_memory_usage(stack_ptr_log_file, stack_ptr_log_file_2, output_plot_path, args.title,
                                   args.subtitle1, args.subtitle2, not args.noshow)
    else:
        plot_stack_memory_usage(stack_ptr_log_file, output_plot_path, args.title, not args.noshow)


if __name__ == "__main__":
    # parse CLI args
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(required=True)

    parser_stack = subparsers.add_parser("stack", help="Stack memory usage profiling")
    parser_stack.add_argument("logfile", help="Path to stack pointer log file")
    parser_stack.add_argument("logfile2", default="", nargs="?",
                              help="Path to second stack pointer log file, when using --compare")
    parser_stack.add_argument("--compare", "-c", action="store_true", help="Plot two logfiles next to each other")
    parser_stack.add_argument("--output", "-o", help="Path to save plot as PGF file")
    parser_stack.add_argument("--title", default="", help="Plot title")
    parser_stack.add_argument("--subtitle1", default="", help="Title for 1st subplot")
    parser_stack.add_argument("--subtitle2", default="", help="Title for 2nd subplot")
    parser_stack.add_argument("--noshow", action="store_true", help="Don't open the plot window")
    # define the function to call if the stack subcommand is used
    parser_stack.set_defaults(func=stack_memory_usage_profiler)

    args = parser.parse_args()
    args.func(args)
