default:
    just --list

cloc:
    cloc . --by-file --force-lang=Rust,lalrpop --list-file=.cloc

test *ARGS:
    ./tools/testsuite.py {{ARGS}}

run-test *ARGS:
    ./tools/testsuite.py --run {{ARGS}}

run WASM *ARGS:
    ./runtime/run.mjs {{WASM}} {{ARGS}}

compile SOURCE OUTPUT="module.wasm" LOG_LEVEL="error" *ARGS="": build
    RUST_LOG={{LOG_LEVEL}} ./target/debug/c_to_wasm_compiler {{SOURCE}} -o {{OUTPUT}} {{ARGS}}

build:
    cargo build

objdump WASM:
    wasm-objdump {{WASM}} -d

profile-stack *ARGS:
    ./tools/profiler.py stack {{ARGS}}

clear-logs:
    rm ./logs/*.stackptrlog

stack_alloc_unoptimised_title := "\\\"Unoptimised\\\""
stack_alloc_optimised_title := "\\\"Optimised Stack Allocation\\\""
tailcall_unoptimised_title := "\\\"Without Tail-Call Optimisation\\\""
tailcall_optimised_title := "\\\"With Tail-Call Optimisation\\\""

generate-stack-profile-plots:
    # stack allocation policy plots
    just profile-stack --compare --subtitle1 {{stack_alloc_unoptimised_title}} --subtitle2 {{stack_alloc_optimised_title}} --noshow \
        -o ../05-dissertation/21-plots/01-case-compare.pgf \
        logs/noopt-stack-allocation/case.stackptrlog \
        logs/opt-stack-allocation/case.stackptrlog

    just profile-stack --compare --subtitle1 {{stack_alloc_unoptimised_title}} --subtitle2 {{stack_alloc_optimised_title}} --noshow \
        -o ../05-dissertation/21-plots/02-fibonacci-compare.pgf \
        logs/noopt-stack-allocation/fibonacci.stackptrlog \
        logs/opt-stack-allocation/fibonacci.stackptrlog

    just profile-stack --compare --subtitle1 {{stack_alloc_unoptimised_title}} --subtitle2 {{stack_alloc_optimised_title}} --noshow \
        -o ../05-dissertation/21-plots/03-gameoflife-blinker-compare.pgf \
        logs/noopt-stack-allocation/gameoflife-blinker.stackptrlog \
        logs/opt-stack-allocation/gameoflife-blinker.stackptrlog

    just profile-stack --compare --subtitle1 {{stack_alloc_unoptimised_title}} --subtitle2 {{stack_alloc_optimised_title}} --noshow \
        -o ../05-dissertation/21-plots/04-gameoflife-pulsar-compare.pgf \
        logs/noopt-stack-allocation/gameoflife-pulsar.stackptrlog \
        logs/opt-stack-allocation/gameoflife-pulsar.stackptrlog

    just profile-stack --compare --subtitle1 {{stack_alloc_unoptimised_title}} --subtitle2 {{stack_alloc_optimised_title}} --noshow \
        -o ../05-dissertation/21-plots/05-gcd-compare.pgf \
        logs/noopt-stack-allocation/gcd.stackptrlog \
        logs/opt-stack-allocation/gcd.stackptrlog

    just profile-stack --compare --subtitle1 {{stack_alloc_unoptimised_title}} --subtitle2 {{stack_alloc_optimised_title}} --noshow \
        -o ../05-dissertation/21-plots/06-hexify-compare.pgf \
        logs/noopt-stack-allocation/hexify.stackptrlog \
        logs/opt-stack-allocation/hexify.stackptrlog

    just profile-stack --compare --subtitle1 {{stack_alloc_unoptimised_title}} --subtitle2 {{stack_alloc_optimised_title}} --noshow \
        -o ../05-dissertation/21-plots/07-occurrences-compare.pgf \
        logs/noopt-stack-allocation/occurrences.stackptrlog \
        logs/opt-stack-allocation/occurrences.stackptrlog

    just profile-stack --compare --subtitle1 {{stack_alloc_unoptimised_title}} --subtitle2 {{stack_alloc_optimised_title}} --noshow \
        -o ../05-dissertation/21-plots/08-strlen-compare.pgf \
        logs/noopt-stack-allocation/strlen.stackptrlog \
        logs/opt-stack-allocation/strlen.stackptrlog

    just profile-stack --compare --subtitle1 {{stack_alloc_unoptimised_title}} --subtitle2 {{stack_alloc_optimised_title}} --noshow \
        -o ../05-dissertation/21-plots/09-wildcardcmp-compare.pgf \
        logs/noopt-stack-allocation/wildcardcmp.stackptrlog \
        logs/opt-stack-allocation/wildcardcmp.stackptrlog

    just profile-stack --compare --subtitle1 {{stack_alloc_unoptimised_title}} --subtitle2 {{stack_alloc_optimised_title}} --noshow \
        -o ../05-dissertation/21-plots/10-trim-compare.pgf \
        logs/noopt-stack-allocation/trim.stackptrlog \
        logs/opt-stack-allocation/trim.stackptrlog

    just profile-stack --compare --subtitle1 {{stack_alloc_unoptimised_title}} --subtitle2 {{stack_alloc_optimised_title}} --noshow \
        -o ../05-dissertation/21-plots/11-tailcall-sum-compare.pgf \
        logs/noopt-stack-allocation/tailcall-sum.stackptrlog \
        logs/opt-stack-allocation/tailcall-sum.stackptrlog

    just profile-stack --compare --subtitle1 {{stack_alloc_unoptimised_title}} --subtitle2 {{stack_alloc_optimised_title}} --noshow \
        -o ../05-dissertation/21-plots/12-tailcall-sum-compare-without-tailcallopt.pgf \
        logs/without-tailcallopt/noopt-stack-allocation/tailcall-sum.stackptrlog \
        logs/without-tailcallopt/opt-stack-allocation/tailcall-sum.stackptrlog

    just profile-stack --compare --subtitle1 {{stack_alloc_unoptimised_title}} --subtitle2 {{stack_alloc_optimised_title}} --noshow \
        -o ../05-dissertation/21-plots/13-non-recursive-tailcall-compare-without-tailcallopt.pgf \
        logs/without-tailcallopt/noopt-stack-allocation/non-recursive-tailcall.stackptrlog \
        logs/without-tailcallopt/opt-stack-allocation/non-recursive-tailcall.stackptrlog

    just profile-stack --compare --subtitle1 {{stack_alloc_unoptimised_title}} --subtitle2 {{stack_alloc_optimised_title}} --noshow \
        -o ../05-dissertation/21-plots/14-gcd-compare-without-tailcallopt.pgf \
        logs/without-tailcallopt/noopt-stack-allocation/gcd.stackptrlog \
        logs/without-tailcallopt/opt-stack-allocation/gcd.stackptrlog

    # tail-call optimisation plots
    just profile-stack --compare --subtitle1 {{tailcall_unoptimised_title}} --subtitle2 {{tailcall_optimised_title}} --noshow \
        -o ../05-dissertation/21-plots/22-tailcall-sum-compare-tailcallopt-without-stackopt.pgf \
        logs/compare-tailcallopt/noopt-tailcall/tailcall-sum.stackptrlog \
        logs/compare-tailcallopt/opt-tailcall/tailcall-sum.stackptrlog

    just profile-stack --compare --subtitle1 {{tailcall_unoptimised_title}} --subtitle2 {{tailcall_optimised_title}} --noshow \
        -o ../05-dissertation/21-plots/23-non-recursive-tailcall-compare-tailcallopt-without-stackopt.pgf \
        logs/compare-tailcallopt/noopt-tailcall/non-recursive-tailcall.stackptrlog \
        logs/compare-tailcallopt/opt-tailcall/non-recursive-tailcall.stackptrlog

    just profile-stack --compare --subtitle1 {{tailcall_unoptimised_title}} --subtitle2 {{tailcall_optimised_title}} --noshow \
        -o ../05-dissertation/21-plots/24-gcd-compare-tailcallopt-without-stackopt.pgf \
        logs/compare-tailcallopt/noopt-tailcall/gcd.stackptrlog \
        logs/compare-tailcallopt/opt-tailcall/gcd.stackptrlog

