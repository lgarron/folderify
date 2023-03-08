.PHONY: publish
publish:
	rm -rf ./dist
	python3 setup.py sdist bdist_wheel
	python3 -m twine upload dist/*

.PHONY: test
test:
	./test/test.sh

.PHONY: setup
setup:
	brew install python3
	python3 -m pip install twine
