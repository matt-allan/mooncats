# generate the doc.json used by unit tests
gen-test-data:
  lua-language-server \
    --doc=./testdata \
    --doc-out-path=./testdata --logpath=./testdata