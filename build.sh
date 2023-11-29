#!/bin/sh
#
# Copyright 2023 shadow3aaa@gitbub.com
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
#  You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
SHDIR="$(dirname $(readlink -f "$0"))"
TEMPDIR=$SHDIR/output/.temp
NOARG=true
HELP=false
CLEAN=false
DEBUG_BUILD=false
RELEASE_BUILD=false
VERBOSE=false

init_package() {
	cd $SHDIR
	rm -rf $TEMPDIR
	mkdir -p $TEMPDIR
	cp -rf prebuilt/uperf/* $TEMPDIR
	cp -f prebuilt/inject $TEMPDIR/bin/inject
}

if [ "$TERMUX_VERSION" = "" ]; then
	alias RR='cargo ndk -p 31 -t arm64-v8a'

	if [ "$ANDROID_NDK_HOME" = "" ]; then
		echo Missing ANDROID_NDK_HOME >&2
		exit 1
	else
		dir="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"
		alias strip="$dir/llvm-strip"
	fi
else
	alias RR=cargo
fi

for arg in $@; do
	case $arg in
	clean | --clean)
		CLEAN=true
		;;
	r | -r | release | --release)
		RELEASE_BUILD=true
		;;
	d | -d | debug | --debug)
		DEBUG_BUILD=true
		;;
	-h | h | help | --help)
		HELP=true
		;;
	v | -v | verbose | --verbose)
		VERBOSE=true
		;;
	*)
		echo Illegal parameter: $arg >&2
		echo Try \'./build.sh --help\' >&2
		exit 1
		;;
	esac

	NOARG=false
done

set -e

if $HELP || $NOARG; then
	echo -n "./build.sh:
    --release / release / -r / r:
        release build
    --debug / debug / -d / d:
        debug build
    --verbose / verbose / -v / v:
        print details of build"

	exit
elif $CLEAN; then
	rm -rf output
	cargo clean

	exit
fi

if $DEBUG_BUILD; then
	if $VERBOSE; then
		RR build --target aarch64-linux-android -v
	else
		RR build --target aarch64-linux-android
	fi

	init_package
	cp -f target/aarch64-linux-android/debug/libuperf_patch.so $TEMPDIR/bin/libuperf_patch.so

	cd $TEMPDIR
	zip -9 -rq "../uperf-patched(debug).zip" .

	echo "Module Packaged: output/uperf-patched(debug).zip"
fi

if $RELEASE_BUILD; then
	if $VERBOSE; then
		RR build --release --target aarch64-linux-android -v
	else
		RR build --release --target aarch64-linux-android
	fi

	init_package
	strip $TEMPDIR/bin/libuperf_patch.so
	cp -f target/aarch64-linux-android/release/libuperf_patch.so $TEMPDIR/bin/libuperf_patch.so

	cd $TEMPDIR
	zip -9 -rq "../uperf-patched(release).zip" .

	echo "Module Packaged: output/uperf-patched(release).zip"
fi
