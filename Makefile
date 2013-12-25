SRC := $(wildcard *.rs) $(wildcard behaviour/*.rs)

ron: $(SRC) /usr/bin/rustc
	rustc -o $@ main.rs

clean:
	rm -f ron
