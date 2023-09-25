#!/bin/bash

#set -x
set -e
set -o pipefail
set -u

THIS_SCRIPT=$0

# defaults
DFLT_COMMAND=simple
DFLT_SUBCOMMAND=run
DFLT_RUN_CMD_ARG=inf
DFLT_TARGET=transaction
DFLT_TIMEOUT=1000

function usage() {
    echo "$0 [FUZZER/COMMAND] [SUBCOMMAND] [FUZZ-TARGET] [COMMAND-ARGS]"
    echo "Available targets:"
    echo "    transaction"
    echo "    wasm_instrument"
    echo "    decimal"
    echo "    parse_decimal"
    echo "Available fuzzers"
    echo "    libfuzzer  - 'cargo fuzz' wrapper"
    echo "    afl        - 'cargo afl' wrapper"
    echo "    simple     - simple fuzzer (default)"
    echo "  Subcommands:"
    echo "      init        - Take sample input ('./fuzz_input/<target>') for given test,"
    echo "                    minimize the test corpus and put the result into 'corpus/<target>',"
    echo "                    which is used by 'libfuzzer' as initial input."
    echo "                    Applicable only for 'libfuzzer'."
    echo "      build       - Build fuzz test for given fuzzer."
    echo "                    Binaries are built in 'release' format."
    echo "      run         - Run fuzz test for given fuzzer (default command)"
    echo "                    It takes arguments that might be supplied to the specified fuzzers."
    echo "                    For more information try:"
    echo "                      $0 [FUZZER] run -h"
    echo "     machine-init - Initialize the OS accordingly"
    echo "                    In case of Linux:"
    echo "                    - disable external utils handling coredumps"
    echo "                    - disable CPU frequency scaling"
    echo "                    Applcable only for 'afl'"
    echo "Available commands"
    echo "    generate-input - generate fuzzing input data"
    echo "  Subcommands:"
    echo "        empty       - Empty input"
    echo "        raw         - Do not process generated data"
    echo "        unique      - Make the input data unique"
    echo "        minimize    - Minimize the input data"
    echo "  Args:"
    echo "        timeout     - timeout in ms"
    echo "Available fuzz targets"
    echo "    transaction"
    echo "Examples:"
    echo "  - build AFL fuzz tests"
    echo "    $0 afl build transaction"
    echo "  - run AFL tests for 1h"
    echo "    $0 afl run transaction -V 3600"
    echo "  - run LibFuzzer for 1h"
    echo "    $0 libfuzzer run transaction -max_total_time=3600"
    echo "  - run simple-fuzzer for 1h"
    echo "    $0 simple run transaction --duration 3600"
    echo "  - reproduce some crash discovered by 'libfuzzer'"
    echo "    $0 libfuzzer run transaction ./artifacts/transaction/crash-ec25d9d2a8c3d401d84da65fd2321cda289d"
    echo "  - reproduce some crash discovered by 'libfuzzer' using 'simple-fuzzer'"
    echo "    RUST_BACKTRACE=1 $0 simple run transaction ./artifacts/transaction/crash-ec25d9d2a8c3d401d84da65fd2321cda289d"
}

function error() {
    local msg=$1
    echo "error - $msg"
    usage
    exit 1
}

function fuzzer_libfuzzer() {
    local cmd=$DFLT_SUBCOMMAND
    if [ $# -ge 1 ] ; then
        cmd=$1
        shift
    fi
    local target=$DFLT_TARGET
    if [ $# -ge 1 ] ; then
        target=$1
        shift
    fi
    local run_args=""
    if [ "$cmd" = "run" ] ; then
        run_args="$@"

    elif [ "$cmd" = "init" ] ; then
        # initial setup:
        # - minimize the corpus:
        #    https://llvm.org/docs/LibFuzzer.html#id25
        #
        #   cargo +nightly fuzz $target  --fuzz-dir radix-engine-fuzz \
        #      --no-cfg-fuzzing --target-dir target-libfuzzer $target -- \
        #      -merge=1 corpus/$target <INTERESTING_INPUTS_DIR/FULL_CORPUS_DIR>
        #
        cmd=run
        run_args="-- -merge=1 corpus/${target} fuzz_input/${target} "
    fi
    # Unset cfg=fuzzing by --no-cfg-fuzzing.
    # "secp256k1" uses some stubs instead of true cryptography if "fuzzing" is set.
    # see: https://github.com/rust-bitcoin/rust-secp256k1/#fuzzing
    set -x
    cargo +nightly fuzz $cmd \
        --release \
        --no-default-features --features std,libfuzzer-sys,post_run_db_check\
        --fuzz-dir . \
        --no-cfg-fuzzing \
        --target-dir target-libfuzzer \
        $target \
        $run_args

}

function fuzzer_afl() {
    local cmd=$DFLT_SUBCOMMAND
    if [ $# -ge 1 ] ; then
        cmd=$1
        shift
    fi
    local target=$DFLT_TARGET
    if [ $# -ge 1 ] ; then
        target=$1
        shift
    fi

    if [ $cmd = "build" ] ; then
        set -x
        cargo afl build --release \
            --bin $target \
            --no-default-features --features std,afl \
            --target-dir target-afl
    elif [ $cmd = "run" ] ; then
        mkdir -p afl/${target}/out
        export AFL_AUTORESUME=1
        set -x
        cargo afl fuzz -i fuzz_input/${target} -o afl/${target} $@ -- target-afl/release/${target}
    elif [ $cmd = "machine-init" ] ; then
        uname="$(uname -s)"
        if [ $uname = "Linux" ] ; then
            set -x
            # disable external utilities handling coredumps
            sudo bash -c "echo core > /proc/sys/kernel/core_pattern"
            # disable CPU frequency scaling
            find /sys/devices/system/cpu -name scaling_governor | \
                xargs -I {} sudo bash -c "echo performance > {}"
        elif [ $uname = "Darwin" ] ; then
            echo "If you see an error message like 'shmget() failed' above, try running the following command:"
            echo "  sudo /Users/<username>/.local/share/afl.rs/rustc-<version>/afl.rs-<version>/afl/bin/afl-system-config"
        else
            error "OS '$uname' not supported"
        fi
    fi
}

function fuzzer_simple() {
    local cmd=$DFLT_SUBCOMMAND
    if [ $# -ge 1 ] ; then
        cmd=$1
        shift
    fi
    local target=$DFLT_TARGET
    if [ $# -ge 1 ] ; then
        target=$1
        shift
    fi

    set -x
    cargo $cmd --release \
        --no-default-features --features std,simple-fuzzer \
        --bin $target \
        -- $@
}

function generate_input() {
    local target=$DFLT_TARGET
    if [ $# -ge 1 ] ; then
        target=$1
        shift
    fi
    # available modes: raw, unique, minimize
    local mode=${1:-minimize}
    local timeout=${2:-$DFLT_TIMEOUT}
    local curr_path=$(pwd)
    local cmin_dir=fuzz_input/${target}_cmin
    local raw_dir=fuzz_input/${target}_raw
    local final_dir=fuzz_input/${target}

    if [ $mode = "empty" ] ; then
        echo "creating empty input $final_dir"
        mkdir -p $final_dir
        # Cannot be empty, let's use newline (0xA).
        echo "" > ${final_dir}/empty
        return
    fi

    if [ $target = "transaction" -o $target = "wasm_instrument" -o $target = "decimal" -o $target = "parse_decimal" ] ; then
        if [ ! -f target-afl/release/${target} ] ; then
            echo "target binary 'target-afl/release/${target}' not built. Call below command to build it"
            echo "$THIS_SCRIPT afl build"
            exit 1
        fi

        mkdir -p $raw_dir $cmin_dir $final_dir
        if [ "$(ls -A ${curr_path}/${raw_dir})" ] ; then
            echo "raw dir is not empty, skipping generation"
            if [ $mode = "raw" ] ; then
                find ${curr_path}/${raw_dir} -type f -name "*" | xargs  -I {} mv {} ${curr_path}/${final_dir}
                return
            fi
        fi


        if [ $target = "transaction" -o $target = "decimal" -o $target = "parse_decimal" ] ; then
            # Collect input data

            cargo nextest run test_${target}_generate_fuzz_input_data  --release

            if [ $mode = "raw" ] ; then
                #mv ../radix-engine-tests/manifest_*.raw ${curr_path}/${final_dir}
                mv ${target}_*.raw ${curr_path}/${final_dir}
                return
            fi

            #mv ../radix-engine-tests/manifest_*.raw ${curr_path}/${raw_dir}
            mv ${target}_*.raw ${curr_path}/${raw_dir}

        elif [ $target = "wasm_instrument" ] ; then
            # TODO generate more wasm inputs. and maybe smaller
            if [ $mode = "raw" ] ; then
                find .. -name   "*.wasm" | while read f ; do cp $f $final_dir ; done
                return
            else
                find .. -name   "*.wasm" | while read f ; do cp $f $raw_dir ; done
            fi
        fi

        # do not minimize big files, move them directly to input
        find ${curr_path}/${raw_dir} -type f -size +100k | xargs -I {} mv "{}" ${curr_path}/${final_dir}

        # Make the input corpus unique
        cargo afl cmin -t $timeout -i $raw_dir -o $cmin_dir -- target-afl/release/${target} 2>&1 | tee afl_cmin.log
        if [ $mode = "unique" ] ; then
            mv $cmin_dir/* $final_dir
            return
        fi

        # if `cargo afl cmin` sets the AFL_MAP_SIZE, then set it also for `cargo afl tmin`
        AFL_MAP_SIZE=$(grep AFL_MAP_SIZE afl_cmin.log | sed -E 's/^.*AFL_MAP_SIZE=//g' || true)
        if [ "$AFL_MAP_SIZE" != "" ] ; then
            export AFL_MAP_SIZE
        fi

        # Minimize all corpus files
        pushd $cmin_dir
        # Filter out the files not greater than 100k to reduce minimizing duration
        if which parallel && parallel --version | grep -q 'GNU parallel' ; then
            # parallel is nicer because is serializes output from commands in parallel.
            # "halt now,fail=1" - exit when any job has failed. Kill other running jobs
            find . -type f -size -100k | parallel --halt now,fail=1 -- \
                cargo afl tmin -t $timeout -i "{}" -o "${curr_path}/${final_dir}/{/}" -- ${curr_path}/target-afl/release/${target}
        else
            find . -type f -size -100k | xargs -P 8 -I {} \
                cargo afl tmin -t $timeout -i "{}" -o "${curr_path}/${final_dir}/{}" -- ${curr_path}/target-afl/release/${target}
        fi
        popd
    else
        echo "error: target '$target' not supported"
        exit 1
    fi
}

if [ $# -ge 1 ] ; then
    # available fuzzers/commands: libfuzzer, afl, simple, generate-input
    cmd=$1
    shift
else
    cmd=$DFLT_COMMAND
fi

if [ $# -eq 0 ] ; then
    usage
fi

if [ $cmd = "libfuzzer" ] ; then
    fuzzer_libfuzzer $@
elif [ $cmd = "afl" ] ; then
    fuzzer_afl $@
elif [ $cmd = "simple" ] ; then
    fuzzer_simple $@
elif [ $cmd = "generate-input" ] ; then
    generate_input $@
else
    if [ $cmd != "help" -a $cmd != "h" ] ; then
        error "invalid command '$cmd' specified"
    fi
    usage
fi
