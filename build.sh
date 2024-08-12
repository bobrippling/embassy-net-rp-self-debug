#!/bin/sh

usage(){
	echo >&2 "Usage: $0 [app|bootloader]"
	exit 2
}

build_app(){
	cargo build --bin embassy-net-rp-self-debug
}

build_bootloader(){
	cp bootloader.x memory.x
	export PRESERVE_MEMORY_X=1
	cargo build --bin bootloader
}

case $# in
	0)
		build_app
		;;
	1)
		case "$1" in
			app) build_app ;;
			bootloader) build_bootloader ;;
			*) usage ;;
		esac
		;;
	*)
		usage
		;;
esac
