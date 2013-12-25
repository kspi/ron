SRC := $(wildcard *.rs) $(wildcard behaviour/*.rs)

main: $(SRC) /usr/bin/rustc
	rustc main.rs
