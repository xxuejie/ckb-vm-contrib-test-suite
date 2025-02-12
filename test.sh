#!/bin/bash
set -ex

if [ ! $(command -v "riscv64-ckb-elf-gcc") ]
then
  echo "Please install ckb-contract-toolchain!."
  exit 1
fi

# Inspired from https://stackoverflow.com/a/246128
TOP="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
cd $TOP

RUNTESTS=1
if [ "$1" == "--build-only" ]
then
  RUNTESTS=0
  shift
fi

# Prebuilt prefix allows us to do cross-compile outside of the target environment, saving time in qemu setup.
if [ "$1" = "--prebuilt-prefix" ]
then
  shift
  PREBUILT_PREFIX="$1"
  shift
fi

# If requested, make sure we are using latest revision of CKB VM
if [ "$1" = "--update-ckb-vm-contrib" ]
then
    rm -rf ckb-vm-contrib
    shift
fi

if [ ! -d "$TOP/ckb-vm-contrib" ]
then
    git clone https://github.com/xxuejie/ckb-vm-contrib "$TOP/ckb-vm-contrib"
fi

if [ "$RUNTESTS" -eq "1" ]
then
if [ "$1" = "--coverage" ]
then
    AST_INTERPRETER32="kcov --verify $TOP/coverage $TOP/binary/target/$PREBUILT_PREFIX/debug/ast_interpreter32"
    AST_INTERPRETER64="kcov --verify $TOP/coverage $TOP/binary/target/$PREBUILT_PREFIX/debug/ast_interpreter64"
    LLVM_AOT64="kcov --verify $TOP/coverage $TOP/binary/target/$PREBUILT_PREFIX/debug/llvm_aot64"
    DISASSEMBLER32="kcov --verify $TOP/coverage $TOP/binary/target/$PREBUILT_PREFIX/debug/disassembler32"
    DISASSEMBLER64="kcov --verify $TOP/coverage $TOP/binary/target/$PREBUILT_PREFIX/debug/disassembler64"

    rm -rf $TOP/coverage

    if [ "x$PREBUILT_PREFIX" = "x" ]
    then
        # Build CKB VM binaries for testing
        cd "$TOP/binary"
        cargo build $BUILD_OPTIONS
    fi
else
    AST_INTERPRETER32="$TOP/binary/target/$PREBUILT_PREFIX/release/ast_interpreter32"
    AST_INTERPRETER64="$TOP/binary/target/$PREBUILT_PREFIX/release/ast_interpreter64"
    LLVM_AOT64="$TOP/binary/target/$PREBUILT_PREFIX/release/llvm_aot64"
    DISASSEMBLER32="$TOP/binary/target/$PREBUILT_PREFIX/release/disassembler32"
    DISASSEMBLER64="$TOP/binary/target/$PREBUILT_PREFIX/release/disassembler64"

    if [ "x$PREBUILT_PREFIX" = "x" ]
    then
        # Build CKB VM binaries for testing
        cd "$TOP/binary"
        cargo build --release $BUILD_OPTIONS
    fi
fi
fi

# Tests from ckb-vm
cd "$TOP"
CKB_VM_PREFIX="ckb-vm-for-tests/tests/programs/"
CKB_VM_TEST_PROGRAMS=$(cat ckb_vm_programs.txt)

for p in $CKB_VM_TEST_PROGRAMS; do
  $AST_INTERPRETER64 ${CKB_VM_PREFIX}${p}
done

for p in $CKB_VM_TEST_PROGRAMS; do
  $LLVM_AOT64 ${CKB_VM_PREFIX}${p}
done

for p in $CKB_VM_TEST_PROGRAMS; do
  $DISASSEMBLER64 ${CKB_VM_PREFIX}${p}
done

# Build riscv-tests
cd "$TOP/riscv-tests"
autoconf
./configure --target=riscv64-ckb-elf
make isa

if [ "$RUNTESTS" -eq "1" ]
then
    # Test CKB VM with riscv-tests
    # NOTE: let's stick with the simple way here since we know there won't be
    # whitespaces, otherwise shell might not be a good option here.
    for i in $(find . -regex ".*/rv32u[imc]-u-[a-z0-9_]*" | grep -v "fence_i"); do
        $AST_INTERPRETER32 $i
    done
    for i in $(find . -regex ".*/rv64u[imc]-u-[a-z0-9_]*" | grep -v "fence_i"); do
        $AST_INTERPRETER64 $i
    done
    for i in $(find . -regex ".*/rv64u[imc]-u-[a-z0-9_]*" | grep -v "fence_i"); do
        $LLVM_AOT64 $i
    done
    for i in $(find . -regex ".*/rv32u[imc]-u-[a-z0-9_]*" | grep -v "fence_i"); do
        $DISASSEMBLER32 $i
    done
    for i in $(find . -regex ".*/rv64u[imc]-u-[a-z0-9_]*" | grep -v "fence_i"); do
        $DISASSEMBLER64 $i
    done
fi

# Test CKB VM with riscv-compliance
cd "$TOP/riscv-compliance"

if [ "$RUNTESTS" -eq "1" ]
then
    COMPLIANCE_TARGET="simulate"
else
    COMPLIANCE_TARGET="compile"
fi

# TODO: more targets
mkdir -p work
find work -name "*.log" -delete && make RISCV_PREFIX=riscv64-ckb-elf- RISCV_TARGET=ckb-vm XLEN=64 RISCV_DEVICE=I TARGET_SIM="$AST_INTERPRETER64" $COMPLIANCE_TARGET
find work -name "*.log" -delete && make RISCV_PREFIX=riscv64-ckb-elf- RISCV_TARGET=ckb-vm XLEN=64 RISCV_DEVICE=M TARGET_SIM="$AST_INTERPRETER64" $COMPLIANCE_TARGET
find work -name "*.log" -delete && make RISCV_PREFIX=riscv64-ckb-elf- RISCV_TARGET=ckb-vm XLEN=64 RISCV_DEVICE=C TARGET_SIM="$AST_INTERPRETER64" $COMPLIANCE_TARGET

find work -name "*.log" -delete && make RISCV_PREFIX=riscv64-ckb-elf- RISCV_TARGET=ckb-vm XLEN=64 RISCV_DEVICE=I TARGET_SIM="$LLVM_AOT64" $COMPLIANCE_TARGET
find work -name "*.log" -delete && make RISCV_PREFIX=riscv64-ckb-elf- RISCV_TARGET=ckb-vm XLEN=64 RISCV_DEVICE=M TARGET_SIM="$LLVM_AOT64" $COMPLIANCE_TARGET
find work -name "*.log" -delete && make RISCV_PREFIX=riscv64-ckb-elf- RISCV_TARGET=ckb-vm XLEN=64 RISCV_DEVICE=C TARGET_SIM="$LLVM_AOT64" $COMPLIANCE_TARGET

find work -name "*.log" -delete && make RISCV_PREFIX=riscv64-ckb-elf- RISCV_TARGET=ckb-vm XLEN=64 RISCV_DEVICE=I TARGET_SIM="$DISASSEMBLER64" $COMPLIANCE_TARGET
find work -name "*.log" -delete && make RISCV_PREFIX=riscv64-ckb-elf- RISCV_TARGET=ckb-vm XLEN=64 RISCV_DEVICE=M TARGET_SIM="$DISASSEMBLER64" $COMPLIANCE_TARGET
find work -name "*.log" -delete && make RISCV_PREFIX=riscv64-ckb-elf- RISCV_TARGET=ckb-vm XLEN=64 RISCV_DEVICE=C TARGET_SIM="$DISASSEMBLER64" $COMPLIANCE_TARGET

# Even though ckb-vm-bench-scripts are mainly used for benchmarks, they also
# contains sophisticated scripts which make good tests
cd "$TOP/ckb-vm-bench-scripts"
make CC=riscv64-ckb-elf-gcc DUMP=riscv64-ckb-elf-objdump HOST=riscv64-ckb-elf

if [ "$RUNTESTS" -eq "1" ]
then
    $AST_INTERPRETER64 build/secp256k1_bench 033f8cf9c4d51a33206a6c1c6b27d2cc5129daa19dbd1fc148d395284f6b26411f 304402203679d909f43f073c7c1dcf8468a485090589079ee834e6eed92fea9b09b06a2402201e46f1075afa18f306715e7db87493e7b7e779569aa13c64ab3d09980b3560a3 foo bar
    $AST_INTERPRETER64 build/schnorr_bench 4103c5b538d6f695a961e916e7308211c8c917e1e02ca28a21b0989596a9ffb6 e45408b5981ec7fd6e72faa161776fe5db17dd92226d1ad784816fb843e151127d9ccb615f364f317a35e2ddddc91bbf30ad103ddfd3ad7e839f508dbfe6298a foo bar

    $LLVM_AOT64 build/secp256k1_bench 033f8cf9c4d51a33206a6c1c6b27d2cc5129daa19dbd1fc148d395284f6b26411f 304402203679d909f43f073c7c1dcf8468a485090589079ee834e6eed92fea9b09b06a2402201e46f1075afa18f306715e7db87493e7b7e779569aa13c64ab3d09980b3560a3 foo bar
    $LLVM_AOT64 build/schnorr_bench 4103c5b538d6f695a961e916e7308211c8c917e1e02ca28a21b0989596a9ffb6 e45408b5981ec7fd6e72faa161776fe5db17dd92226d1ad784816fb843e151127d9ccb615f364f317a35e2ddddc91bbf30ad103ddfd3ad7e839f508dbfe6298a foo bar

    $DISASSEMBLER64 build/secp256k1_bench 033f8cf9c4d51a33206a6c1c6b27d2cc5129daa19dbd1fc148d395284f6b26411f 304402203679d909f43f073c7c1dcf8468a485090589079ee834e6eed92fea9b09b06a2402201e46f1075afa18f306715e7db87493e7b7e779569aa13c64ab3d09980b3560a3 foo bar
    $DISASSEMBLER64 build/schnorr_bench 4103c5b538d6f695a961e916e7308211c8c917e1e02ca28a21b0989596a9ffb6 e45408b5981ec7fd6e72faa161776fe5db17dd92226d1ad784816fb843e151127d9ccb615f364f317a35e2ddddc91bbf30ad103ddfd3ad7e839f508dbfe6298a foo bar
fi

echo "All tests are passed!"
