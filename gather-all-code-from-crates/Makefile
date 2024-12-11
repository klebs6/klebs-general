DEFAULT := test

#INCLUDE_TESTS       := --include-tests
EXCLUDE_MAIN        := --exclude-main-file
#OMIT_BODIES         := --omit-bodies
OMIT_PRIVATE        := --omit-private
#REMOVE_DOC_COMMENTS := --remove-doc-comments

default: $(DEFAULT)

build:
	cargo build

run:
	cargo run -- $(INCLUDE_TESTS) $(EXCLUDE_MAIN) $(OMIT_BODIES) $(OMIT_PRIVATE) $(REMOVE_DOC_COMMENTS)

test:
	cargo test

