#!/usr/bin/env bash

RUNNER=./target/debug/compile
TESTS_DIR=../fonttools/Tests/feaLib/data
OUT_DIR=./compiler_test_out

one_test() {
	p="$TESTS_DIR/$*"
	fea="$p.fea"
	ttf="$p.ttf"
	out="$OUT_DIR/$*.fea-rs.ttf"

	[ ! -f "$fea" ] && {
		echo "$fea" ': no such file or directory'
		exit 1
	}

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
		echo 'need a test file'
		exit 1
		;;

	*)
		[ "$2" == "-f" ] && exec nvim "$TESTS_DIR/$1.fea"
		[ "$2" == "-x" ] && exec nvim "$TESTS_DIR/$1.ttx"
		one_test "$1"
		exit $?
		;;
esac
