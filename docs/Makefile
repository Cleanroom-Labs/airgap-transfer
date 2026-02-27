SPHINXBUILD   = sphinx-build
SOURCEDIR     = source
BUILDDIR      = _build

EXTRACT       = scripts/extract_test_mapping.py
CONVERT       = scripts/convert_test_results.py
MAPPING       = test-mapping.json
TEST_RESULTS  = source/_data/test-results.json

.PHONY: html full test-results test-mapping clean serve

html:
	$(SPHINXBUILD) -b html $(SOURCEDIR) $(BUILDDIR)/html -W

full: test-results html

test-results: test-mapping
	cargo +nightly test -- --format json -Z unstable-options 2>/dev/null \
	  | python3 $(CONVERT) --mapping $(MAPPING) --output $(TEST_RESULTS)

test-mapping:
	python3 $(EXTRACT) --src-dir ../src --test-dir ../tests --output $(MAPPING)

serve: html
	@echo "Serving docs at http://localhost:8000"
	python3 -m http.server 8000 --directory $(BUILDDIR)/html

clean:
	rm -rf $(BUILDDIR)
	rm -f $(TEST_RESULTS) $(MAPPING)
