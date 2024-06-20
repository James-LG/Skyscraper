import jsons
import argparse
import sys
import lxml.html

class OutputElement:
    def __init__(self, tag: str, text: str, text_content: str, attrib: dict[str, str], itertext: list[str]):
        self.tag = tag
        self.text = text
        self.text_content = text_content
        self.attrib = attrib
        self.itertext = itertext
    
    def from_lxml_element(element: lxml.html.HtmlElement):
        attributes = {}
        for key, value in element.attrib.items():
            attributes[key] = value

        itertext = list(element.itertext())

        return OutputElement(
            tag=element.tag,
            text=element.text,
            text_content=element.text_content(),
            attrib=attributes,
            itertext=itertext
        )

def test_xpath():
    parser = argparse.ArgumentParser()
    parser.add_argument("xpath", help="XPath to search for")
    
    # add boolean flag to only count the number of elements
    parser.add_argument("-c", "--count-only", action="store_true", help="Only count the number of elements")

    args = parser.parse_args()

    html = ""
    for line in sys.stdin:
        html += line

    tree = lxml.html.fromstring(html)
    results = tree.xpath(args.xpath)

    if args.count_only:
        print(len(results))
        return

    output_list = [OutputElement.from_lxml_element(result) for result in results]
    output = jsons.dumps(output_list, jdkwargs={'indent':4})
    print(output)

if __name__ == "__main__":
    test_xpath()
