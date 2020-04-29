#!/usr/bin/env bash

RUNNER=./target/debug/parse_test
TESTS_DIR=../fonttools/Tests/feaLib/data
ALLTESTS=($TESTS_DIR/*.fea)

pass=()
fail=()

all_tests() {
	echo ''

	count=${#ALLTESTS[@]}

	for ((i = 0; i < count; i++)); do
		printf "\rrunning %d/%d" "$((i + 1))" "$count"
		test=${ALLTESTS[i]}

		if $RUNNER "$test" > /dev/null; then
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

	echo "$p"
	echo ''
	echo '------------------------'
	cat "$p"
	echo '------------------------'
	echo ''

	$RUNNER "$p"
}

case "$1" in
	"")
		all_tests
		;;

	*)
		[ "$2" == "-e" ] && exec nvim "$TESTS_DIR/$1"
		one_test "$1"
		;;
esac
