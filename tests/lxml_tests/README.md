# LXML test project

This project is used to test the accepted behaviour of Xpath expressions.

## Usage

From the root of the repo, you can create a `test.html` file and pipe it into this Python program along with an Xpath expression as an argument.

```sh
cat test.html | python3 tests/lxml_tests/xpath.py "//div[contains(text(),//div[#id='select']/@class)]"
```
