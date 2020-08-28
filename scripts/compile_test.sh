#!/usr/bin/env bash

RUNNER=./target/debug/compile
TESTS_DIR=../fonttools/Tests/feaLib/data
OUT_DIR=./compiler_test_out
ALLTESTS=($TESTS_DIR/*.fea)

mkdir -p $OUT_DIR

all_tests() {
	echo ''
	cargo build
	echo ''

	count=${#ALLTESTS[@]}

	for ((i = 0; i < count; i++)); do
		printf "\rrunning %d/%d" "$((i + 1))" "$count"

		test=${ALLTESTS[i]}
		out="$OUT_DIR/$(basename "${test%%.fea}").fea-rs.ttf"

		if $RUNNER "$test" "$out" > /dev/null 2>&1; then
			pass+=("$test")
		else
			fail+=("$test")
		fi
	done

	printf '\r                             \r'

	echo 'passing:'
	for test in "${pass[@]}"; do
		echo "    ${test##*/}"
	done

	echo ''

	echo 'failing:'
	for test in "${fail[@]}"; do
		echo "    ${test##*/}"
	done

	echo ''
	echo ''
	echo "total test cases: $count"
	echo "  passing: ${#pass[@]}"
	echo "  failing: ${#fail[@]}"
	echo ''
}

one_test() {
	p="$TESTS_DIR/$*"
	fea="$p.fea"
	ttf="$p.ttf"
	out="$OUT_DIR/$*.fea-rs.ttf"

	[ ! -f "$fea" ] && {
		echo "$fea" ': no such file or directory'
		exit 1
	}

	echo ''
	cargo build
	echo ''

	echo "$fea"
	echo ''
	echo '------------------------'
	cat "$fea"
	echo '------------------------'
	echo ''

	if $RUNNER "$fea" "$out"; then
		if diff "$out" "$ttf"; then
			echo 'pass!';
			return 0
		fi
	fi
	echo 'fail!'
	return 1
}

case "$1" in
	"")
		all_tests
		;;

	*)
		[ "$2" == "-f" ] && exec nvim "$TESTS_DIR/$1.fea"
		[ "$2" == "-x" ] && exec nvim "$TESTS_DIR/$1.ttx"
		one_test "$1"
		exit $?
		;;
esac
