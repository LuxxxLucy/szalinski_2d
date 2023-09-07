#!/usr/bin/env bash
# main.sh: main entry for the programs.
# Author: Jialin Lu luxxxlucy@gmail.com

# Unofficial Bash Strict Mode
set -euo pipefail

log_err() {
  echo "[$(date +'%Y-%m-%dT%H:%M:%S%z')]: $*" >&2
}

function build {
    echo "=== Building: $@" 
    declare executable=( "parse-csg" "optimize")
    for bin in "${executable[@]}"
    do
        echo "bin  : $bin"
        [[ -f $BUILD_DIR/$bin ]] || { echo "Building $bin ..." && cargo build "CARGO_RELEASE_ARG" --bin $bin && echo "Building $bin success" ; }
    done
}

function preprocessing {
    echo "$FUNCNAME: transforming csg to cs expression"
    echo "$# arguments are supplied. Input: $1 Output: $2"
    SZ_PARSE_SHUFFLE_RNG=0 $parse_csg $1 $2
}

function refine {
    echo "$FUNCNAME: optimize the expression "
    echo "$# arguments are supplied. Input: $1 Output: $2"
    [[ ! -e $RUN_PARAMS ]] && echo "Parameter $RUN_PARAMS does not exit" && exit 1
    export $(cat $RUN_PARAMS | xargs) && $optimize $1 $2
}

function run {
    echo "=== Testing"

    file_dir=out/aec-table2
    file_name=CardFramer
    
    csg_file=$file_dir/$file_name.fn.csg
    csexp_file=$file_dir/$file_name.fn.csexp
    cs_file=$file_dir/$file_name.fn.cs

    parse_csg=$BUILD_DIR/parse-csg
    optimize=$BUILD_DIR/optimize

    preprocessing $csg_file $csexp_file
    refine $csexp_file $cs_file
}

function consume_opt {
    local -i count=0
    while getopts ':-:dv' VAL ; do
        case $VAL in
            d ) DEBUG=on ;;
            v ) 
                VERBOSE=on
                set -x
                ;;
            #--------------------------------------------------------
            - )
                 case $OPTARG in
                    debug    ) DEBUG=on ;;
                    verbose  ) VERBOSE=on ;;
                    params=* ) RUN_PARAMS="${OPTARG#*=}" ;;
                    params   )
                            RUN_PARAMS="${!OPTIND}"
                            count=$((count++))
                    ;;
                    * )
                        log_err "error: unknown option: $OPTARG"
                        exit 1
                    ;;
                esac
            ;;
            #--------------------------------------------------------
            : ) log_err "error: no argument supplied" ;;
            * )
                log_err "error: unknown option $OPTARG"
                exit 1
            ;;
        esac
        count=$((count++))
    done

    if [[ $DEBUG == "on" ]] ; then
        BUILD_DIR=target/debug
        CARGO_RELEASE_ARG=
    else
        BUILD_DIR=target/release
        CARGO_RELEASE_ARG=--release
    fi

    shift_num=$count
}

PROGRAM=${0##*/}  # bash version of `basename`
PROGRAM_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

VERBOSE=off  # Default is off
DEBUG=off    # Default is off

RUN_PARAMS=$PROGRAM_DIR/default_params

action="$1"
shift

echo "start. begining of $PROGRAM"

case "$action" in 
    ###############################  Basic
    build )             # Building
        consume_opt $@
        shift $shift_num
        build $@
    ;;
    run )              # Testing
        consume_opt $@
        shift $shift_num
        run 
    ;;
    cleanup )           ## Clean up all artifact
        rm -fv $BUILD_DIR
    ;;

    * )
        ( echo "Usage:"
        egrep '\)[[:space:]]+# '   $0
        echo ''
        egrep '\)[[:space:]]+## '  $0
        echo ''
        egrep '\)[[:space:]]+### ' $0 ) | grep "${1:-.}" | more
    ;;
esac

echo "success. End of $PROGRAM"

