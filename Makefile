SRC := $(wildcard *.rs) $(wildcard behaviour/*.rs)

ron: $(SRC) /usr/bin/rustc
	rustc -Lncurses-rs/lib -o $@ main.rs

clean:
	rm -f ron
