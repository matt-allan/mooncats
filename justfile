# generate the doc.json used by unit tests
gen-test-data:
  lua-language-server \
    --doc=./testdata \
    --doc-out-path=./testdata --logpath=./testdata

serve-test-book:
  RUST_BACKTRACE=1 RUST_LOG=debug mdbook serve ./test_book