#
default: test

#
$(VERBOSE).SILENT:

build test update:
	cargo $@
example-%:
	cargo run --example $*

clean: rm-autosave rm-beam

distclean: rm-lock
	cargo clean

#
rm-autosave:
	find . -name "*~" | xargs rm -f
rm-beam:
	find . -name "*.beam" | xargs rm -f
rm-lock:
	rm -f *.lock
